use std::sync::atomic::Ordering;

use ike::{
    core::{stage, CommandQueue},
    prelude::*,
    reflect::{ReflectComponent, ReflectInspect, ReflectMut, Struct},
    render::RenderSurface,
};
use ike_egui::{
    egui::{
        self, popup, CentralPanel, CtxRef, Response, ScrollArea, SidePanel, TopBottomPanel, Ui,
    },
    EguiTexture, EguiTextures,
};

use crate::load_app::LoadedApp;

#[derive(Hash)]
pub struct MainTexture;

impl EguiTexture for MainTexture {
    fn get(&self, world: &WorldRef) -> Option<wgpu::TextureView> {
        let app = world.get_resource::<LoadedApp>()?;

        let mut render_surface =
            unsafe { app.app.world().resources().write_named::<RenderSurface>()? };

        Some(render_surface.texture()?.create_view(&Default::default()))
    }
}

#[derive(Default)]
pub struct Inspector {
    pub selected: Option<Entity>,
}

pub fn ui_system(
    mut loaded: ResMut<LoadedApp>,
    mut inspector: ResMut<Inspector>,
    egui_textures: Res<EguiTextures>,
    egui_ctx: Res<CtxRef>,
) {
    TopBottomPanel::top("top_panel").show(&egui_ctx, |ui| {
        let _ = ui.button("File");

        if ui.input().modifiers.ctrl && ui.input().key_pressed(egui::Key::S) {
            let scene = loaded.app.world_mut().world_ref(|world_ref| {
                let type_registry = unsafe {
                    world_ref
                        .world()
                        .resources()
                        .read_named::<TypeRegistry>()
                        .unwrap()
                };

                Scene::from_world(&world_ref, &type_registry)
            });

            let scene_str = ron::ser::to_string_pretty(&scene, Default::default()).unwrap();

            std::fs::write("scene.scn", scene_str).unwrap();
        }
    });

    SidePanel::left("left_panel").show(&egui_ctx, |ui| {
        for (entity, name) in loaded.app.world().nodes() {
            if ui.button(name).clicked() {
                inspector.selected = Some(*entity);
            }
        }

        if ui.button("+").clicked() {
            let entity = loaded.app.world().entities().reserve_entity();
            loaded.app.world_mut().entities_mut().spawn(entity);
            loaded.app.world_mut().set_node_name(&entity, "new entity");
        }
    });

    if let Some(ref selected) = inspector.selected {
        SidePanel::left("inspector_panel").show(&egui_ctx, |ui| {
            if let Some(name) = loaded.app.world_mut().get_node_name_mut(selected) {
                ui.text_edit_singleline(name);
            }

            let mut command_queue = CommandQueue::default();
            let commands = Commands::new(loaded.app.world().entities(), &mut command_queue);

            let type_registry = unsafe {
                loaded
                    .app
                    .world()
                    .resources()
                    .read_named::<TypeRegistry>()
                    .unwrap()
            };

            ScrollArea::vertical().show(ui, |ui| {
                for (type_id, registration) in type_registry.iter() {
                    if let Some(reflect_component) =
                        unsafe { registration.data_named::<ReflectComponent>() }
                    {
                        if let Some(storage) = loaded.app.world().entities().storage_raw(type_id) {
                            if let Some(component) =
                                unsafe { reflect_component.from_storage_mut(storage, selected) }
                            {
                                ui.separator();

                                ui.horizontal(|ui| {
                                    ui.label(registration.short_name());

                                    if ui.button("-").clicked() {
                                        commands.remove_component_raw(selected, type_id);
                                    }
                                });

                                ui.separator();

                                let response = reflect_ui(
                                    component,
                                    ui,
                                    &type_registry,
                                    loaded.app.world().resources(),
                                );

                                if response.map_or(false, |r| r.changed()) {
                                    let changed = storage.get_change_marker(selected).unwrap();

                                    changed
                                        .store(loaded.app.world().change_tick(), Ordering::Release);
                                }
                            }
                        }
                    }
                }
            });

            ui.separator();

            let response = ui.button("+");
            let popup_id = ui.make_persistent_id("component_add_popup");
            if response.clicked() {
                ui.memory().toggle_popup(popup_id);
            }

            popup::popup_below_widget(ui, popup_id, &response, |ui| {
                ui.set_max_width(300.0);

                ScrollArea::vertical().show(ui, |ui| {
                    for (type_id, registration) in type_registry.iter() {
                        if let Some(reflect_component) =
                            unsafe { registration.data_named::<ReflectComponent>() }
                        {
                            if ui.button(registration.short_name()).clicked() {
                                let component = reflect_component.default();
                                reflect_component.insert(selected, component.as_ref(), &commands)
                            }
                        }
                    }
                });
            });

            drop(type_registry);
            command_queue.apply(loaded.app.world_mut());
        });
    }

    CentralPanel::default().show(&egui_ctx, |ui| {
        let available = ui.available_size();

        let size = UVec2::new(available.x.ceil() as u32, available.y.ceil() as u32);

        // resize app render surface
        {
            let render_surface = unsafe {
                loaded
                    .app
                    .world()
                    .resources()
                    .write_named::<RenderSurface>()
            };

            if let Some(mut render_surface) = render_surface {
                if render_surface.size() != size {
                    render_surface.configure().width = size.x;
                    render_surface.configure().height = size.y;
                }
            }
        }

        loaded.app.execute_stage(stage::MAINTAIN);
        loaded.app.execute_stage(stage::PRE_RENDER);
        loaded.app.execute_stage(stage::RENDER);
        loaded.app.execute_stage(stage::POST_RENDER);

        let id = egui_textures.get_id(&MainTexture).unwrap();

        ui.image(id, available);
    });
}

fn combine_response(response: &mut Option<Response>, other: Option<Response>) {
    if let Some(response) = response {
        if let Some(other) = other {
            *response |= other;
        }
    } else {
        if other.is_some() {
            *response = other;
        }
    }
}

fn reflect_ui(
    reflect: &mut dyn Reflect,
    ui: &mut Ui,
    type_registry: &TypeRegistry,
    resources: &Resources,
) -> Option<Response> {
    if let Some(registration) = type_registry.get_name(reflect.type_name()) {
        if let Some(inspect) = unsafe { registration.data_named::<ReflectInspect>() } {
            return inspect.inspect(reflect.any_mut(), ui, resources);
        }
    }

    match reflect.reflect_mut() {
        ReflectMut::Struct(value) => reflect_ui_struct(value, ui, type_registry, resources),
        _ => None,
    }
}

fn reflect_ui_struct(
    value: &mut dyn Struct,
    ui: &mut Ui,
    type_registry: &TypeRegistry,
    resources: &Resources,
) -> Option<Response> {
    let mut response = None;

    for i in 0..value.field_len() {
        let name = value.name_at(i).unwrap().to_owned();
        let value = value.field_at_mut(i).unwrap();
        ui.label(&name);
        ui.indent(&name, |ui| {
            let other = reflect_ui(value, ui, type_registry, resources);
            combine_response(&mut response, other);
        });
    }

    response
}
