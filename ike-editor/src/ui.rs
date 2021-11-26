use std::sync::atomic::Ordering;

use ike::{
    core::{stage, CommandQueue, DynamicApp},
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

use crate::{file_browser::FileBrowser, scenes::Scenes};

#[derive(Hash)]
pub struct MainTexture;

impl EguiTexture for MainTexture {
    fn get(&self, world: &WorldRef) -> Option<wgpu::TextureView> {
        let scenes = world.get_resource::<Scenes>()?;

        if !scenes.is_open() {
            return None;
        }

        let mut render_surface = unsafe {
            scenes
                .current_app()
                .world()
                .resources()
                .write_named::<RenderSurface>()?
        };

        Some(render_surface.texture()?.create_view(&Default::default()))
    }
}

#[derive(Default)]
pub struct Inspector {
    pub selected: Option<Entity>,
}

pub fn ui_system(
    mut scenes: ResMut<Scenes>,
    mut inspector: ResMut<Inspector>,
    mut file_browser: ResMut<FileBrowser>,
    egui_textures: Res<EguiTextures>,
    egui_ctx: Res<CtxRef>,
) {
    top_panel(&egui_ctx);

    left_panel(&egui_ctx, &mut scenes, &mut inspector, &mut file_browser);

    if !scenes.is_open() {
        return;
    }

    let app = scenes.current_app_mut();
    let mut command_queue = CommandQueue::default();
    let commands = &Commands::new(app.world().entities(), &mut command_queue);

    inspector_panel(&egui_ctx, app, &inspector, &commands);

    scene_select_panel(&egui_ctx, &mut scenes);

    let app = scenes.current_app_mut();
    scene_view_panel(&egui_ctx, app, &egui_textures);

    command_queue.apply(app.world_mut());
}

fn top_panel(ctx: &CtxRef) {
    TopBottomPanel::top("top_panel").show(ctx, |ui| {
        let _ = ui.button("File");
    });
}

fn left_panel(
    ctx: &CtxRef,
    scenes: &mut Scenes,
    inspector: &mut Inspector,
    file_browser: &mut FileBrowser,
) {
    SidePanel::left("left_panel").show(ctx, |ui| {
        let height = ui.available_height();

        if scenes.is_open() {
            let app = scenes.current_app_mut();

            let mut command_queue = CommandQueue::default();
            let commands = &Commands::new(app.world().entities(), &mut command_queue);

            ScrollArea::vertical()
                .id_source("entity_browser")
                .max_height(height / 2.0)
                .show(ui, |ui| {
                    for (entity, name) in app.world().nodes() {
                        ui.horizontal(|ui| {
                            ui.selectable_value(&mut inspector.selected, Some(*entity), name);

                            if ui.button("-").clicked() {
                                commands.despawn(entity);
                            }
                        });
                    }

                    if ui.button("+").clicked() {
                        commands.spawn_node("new entity");
                    }
                });

            command_queue.apply(app.world_mut());

            ui.separator();
        }

        ScrollArea::vertical()
            .id_source("file_browser")
            .show(ui, |ui| {
                file_browser.ui(ui, scenes).unwrap();
            });
    });
}

fn inspector_panel(ctx: &CtxRef, app: &DynamicApp, inspector: &Inspector, commands: &Commands) {
    if let Some(ref selected) = inspector.selected {
        SidePanel::left("inspector_panel").show(ctx, |ui| {
            if let Some(mut name) = app.world().get_node_name(selected).cloned() {
                let response = ui.text_edit_singleline(&mut name);

                if response.changed() {
                    commands.set_node_name(selected, name);
                }
            }

            let type_registry = unsafe {
                app.world()
                    .resources()
                    .read_named::<TypeRegistry>()
                    .unwrap()
            };

            ScrollArea::vertical().show(ui, |ui| {
                for (type_id, registration) in type_registry.iter() {
                    if let Some(reflect_component) =
                        unsafe { registration.data_named::<ReflectComponent>() }
                    {
                        if let Some(storage) = app.world().entities().storage_raw(type_id) {
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
                                    app.world().resources(),
                                );

                                if response.map_or(false, |r| r.changed()) {
                                    let changed = storage.get_change_marker(selected).unwrap();

                                    changed.store(app.world().change_tick(), Ordering::Release);
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
        });
    }
}

fn scene_select_panel(ctx: &CtxRef, scenes: &mut Scenes) {
    TopBottomPanel::top("scene_select").show(ctx, |ui| {
        ui.horizontal(|ui| {
            for scene in scenes.scenes.keys() {
                ui.selectable_value(&mut scenes.current, Some(scene.clone()), scene.display());
            }
        });
    });
}

fn scene_view_panel(ctx: &CtxRef, app: &mut DynamicApp, egui_textures: &EguiTextures) {
    CentralPanel::default().show(ctx, |ui| {
        let available = ui.available_size();

        let size = UVec2::new(available.x.ceil() as u32, available.y.ceil() as u32);

        // resize app render surface
        {
            let render_surface = unsafe { app.world().resources().write_named::<RenderSurface>() };

            if let Some(mut render_surface) = render_surface {
                if render_surface.size() != size {
                    render_surface.configure().width = size.x;
                    render_surface.configure().height = size.y;
                }
            }
        }

        app.execute_stage(stage::MAINTAIN);
        app.execute_stage(stage::PRE_RENDER);
        app.execute_stage(stage::RENDER);
        app.execute_stage(stage::POST_RENDER);

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
