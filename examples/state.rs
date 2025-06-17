

fn main() {

}

#[derive(Resource)]
struct Sword(Entity);

enum SwordState {
    IDLE,
    ATTACK
}

fn setup(
    mut commands: Commands
) {
    commands.spawn( (

    )).observe( )
}

fn check_sword_hit(
    sword:Res<Sword>,
    sword_sttate:SwordState,
    query:Query<Sword>,
) {
    if matches!( SwordState::ATTACK, sword_state ) {

    }
}