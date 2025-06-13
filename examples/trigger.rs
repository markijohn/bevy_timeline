
fn main() {
    App::new()
        .add_plugin(TimelinePlugin)
}

fn setup(
    commands:Command,
    timeline:ResMut<Assets<Timeline>>,
) {
    let anim = Timeline::from("mytime.ron").unwrap();
    let anim_session = Timeline::create_session(anim);
    let bind_target = anim_session.bind("Object");

    let entity = comands.spawn( (
        anim_session,
        Timeline::Play
    ) );


}