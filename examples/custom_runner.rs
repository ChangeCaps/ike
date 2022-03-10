use ike::prelude::*;

struct CustomRunner;

impl AppRunner for CustomRunner {
    fn run(self: Box<Self>, mut app: App) {
        println!("starting runner!");

        loop {
            app.update();
        }
    }
}

struct Foo(i32);

impl Component for Foo {
    type Storage = SparseStorage;
}

struct Odd;

impl Component for Odd {
    type Storage = SparseStorage;
}

fn setup(commands: Commands) {
    println!("setup system");

    for i in 0..10 {
        let spawn = commands.spawn();
        spawn.insert(Foo(i));

        if i % 2 == 1 {
            spawn.insert(Odd);
        }
    }
}

fn list_foos(query: Query<&Foo>) {
    //println!("all foos:");

    for foo in query.iter() {
        //println!("foo({})", foo.0);
    }
}

fn list_even_foos(query: Query<Entity, Without<Odd>>) {
    println!("even foos:");

    for foo in query.iter() {
        println!("foo({:?})", foo);
    }
}

fn main() {
    App::new()
        .with_runner(CustomRunner)
        .add_startup_system(setup)
        .add_system(list_foos)
        .add_system(list_even_foos)
        .run();
}
