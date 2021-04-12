#![feature(destructuring_assignment)]
use bevy::core::FixedTimestep;
use bevy::prelude::*;
use itertools::Itertools;
use std::collections::hash_set::HashSet;
use std::collections::LinkedList;

const WIDTH: u32 = 4;
const HEIGHT: u32 = 4;

#[derive(Default, Debug)]
struct Block {
    value: u32,
    merged: bool,
}
#[derive(PartialEq, Clone, Copy, Debug)]
struct Position {
    x: u32,
    y: u32,
}
struct Size {
    x: f32,
    y: f32,
}
#[derive(PartialEq, Clone, Copy, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}
struct SwipeEvent(Direction);
struct BlockColor {
    block0: Handle<ColorMaterial>,
    block1: Handle<ColorMaterial>,
    block2: Handle<ColorMaterial>,
    block3: Handle<ColorMaterial>,
    block4: Handle<ColorMaterial>,
    block5: Handle<ColorMaterial>,
    block6: Handle<ColorMaterial>,
    block7: Handle<ColorMaterial>,
    block8: Handle<ColorMaterial>,
    block9: Handle<ColorMaterial>,
}

fn block_scale(win: Res<Windows>, mut q: Query<(&Size, &mut Sprite), With<Block>>) {
    let win_width = win.get_primary().unwrap().width() as f32;
    let win_height = win.get_primary().unwrap().height() as f32;
    for (sz, mut sprite) in q.iter_mut() {
        let sx = (win_width / WIDTH as f32) * sz.x;
        let sy = (win_height / WIDTH as f32) * sz.y;
        sprite.size = Vec2::new(sx, sy);
    }
}

fn block_pos(win: Res<Windows>, mut q: Query<(&mut Transform, &Position), With<Block>>) {
    let w = win.get_primary().unwrap().width() as f32;
    let h = win.get_primary().unwrap().height() as f32;
    for (mut tx, pos) in q.iter_mut() {
        let x = (pos.x as f32 / WIDTH as f32) * w - w / 2.0 + (w / WIDTH as f32) / 2.0;
        let y = (pos.y as f32 / WIDTH as f32) * h - h / 2.0 + (h / HEIGHT as f32) / 2.0;
        tx.translation = Vec3::new(x, y, 0.0);
    }
}

fn input_events(kbd: Res<Input<KeyCode>>, mut writer: EventWriter<SwipeEvent>) {
    if kbd.just_pressed(KeyCode::W) {
        writer.send(SwipeEvent(Direction::Up))
    } else if kbd.just_pressed(KeyCode::A) {
        writer.send(SwipeEvent(Direction::Left));
    } else if kbd.just_pressed(KeyCode::S) {
        writer.send(SwipeEvent(Direction::Down));
    } else if kbd.just_pressed(KeyCode::D) {
        writer.send(SwipeEvent(Direction::Right));
    }
}

fn block_color(mut q: Query<(&Block, &mut Handle<ColorMaterial>)>, materials: Res<BlockColor>) {
    for (
        Block {
            value: v,
            merged: _,
        },
        mut handle,
    ) in q.iter_mut()
    {
        let col = match *v {
            0 => materials.block0.clone(),
            1 => materials.block1.clone(),
            2 => materials.block2.clone(),
            3 => materials.block3.clone(),
            4 => materials.block4.clone(),
            5 => materials.block5.clone(),
            6 => materials.block6.clone(),
            7 => materials.block7.clone(),
            8 => materials.block8.clone(),
            9 => materials.block9.clone(),
            _ => materials.block9.clone(),
        };
        *handle = col;
    }
}

// Update values and positions along a range (a row or column)
fn swipe_range(
    dir: &Direction,
    v: &mut Vec<(Mut<Block>, Mut<Position>, Entity)>,
    cmd: &mut Commands,
    maxlen: u32,
) {
    println!("vlen = {:?}", v.len());
    if v.len() == 0 {
        return;
    }

    if *dir == Direction::Right || *dir == Direction::Up {
        v.reverse();
    }
    let mut g = v.iter_mut().collect::<LinkedList<_>>();

    let (first_block, first_pos, first_ent) = g.pop_front().unwrap();
    let mut block = first_block;
    let mut pos = first_pos;
    let mut ent = first_ent;
    (pos.x, pos.y) = match dir {
        Direction::Left => (0, pos.y),
        Direction::Right => (maxlen - 1, pos.y),
        Direction::Up => (pos.x, maxlen - 1),
        Direction::Down => (pos.x, 0),
    };
    while let Some((mv_block, mv_pos, mv_ent)) = g.pop_front() {
        if !block.merged && block.value == mv_block.value {
            //
            println!("merge event! ");
            cmd.entity(*ent).despawn();
            (mv_pos.x, mv_pos.y) = (pos.x, pos.y);
            mv_block.value += 1;
        } else {
            (mv_pos.x, mv_pos.y) = match dir {
                Direction::Left => (pos.x + 1, pos.y),
                Direction::Right => (pos.x - 1, pos.y),
                Direction::Up => (pos.x, pos.y - 1),
                Direction::Down => (pos.x, pos.y + 1),
            };
        }
        block = mv_block;
        pos = mv_pos;
        ent = mv_ent;
    }
}

fn block_swipe(
    mut cmd: Commands,
    mut reader: EventReader<SwipeEvent>,
    mut q: Query<(&mut Block, &mut Position, Entity)>,
) {
    for SwipeEvent(dir) in reader.iter() {

        let groupby = q
            .iter_mut()
            .sorted_by_key(|e| match dir {
                Direction::Left | Direction::Right => (e.1.y, e.1.x),
                Direction::Up | Direction::Down => (e.1.x, e.1.y),
            })
            .group_by(|(_, pos, _)| match dir {
                Direction::Up | Direction::Down => pos.x,
                Direction::Left | Direction::Right => pos.y,
            });

        println!("------------------");

        for (i, group) in groupby.into_iter() {
            let mut v = group.collect::<Vec<_>>();
            swipe_range(&dir, &mut v, &mut cmd, HEIGHT);
            println!("{} {:?} {:?}", i, dir, v);

        }

        for (mut block, _, _) in q.iter_mut() {
            block.merged = false;
        }
    }
}

fn game_over(mut q: Query<Entity, With<Block>>, mut cmd: Commands) {
    let max_nb_tiles = (HEIGHT * WIDTH) as usize;
    if q.iter().count() == max_nb_tiles {
        println!("Game over!");
        for e in q.iter_mut() {
            cmd.entity(e).despawn();
        }
        
    }
}

fn setup(mut cmd: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    cmd.spawn_bundle(OrthographicCameraBundle::new_2d());
    cmd.insert_resource(BlockColor {
        block0: materials.add(Color::rgb(0.9, 0.9, 0.9).into()),
        block1: materials.add(Color::rgb(0.8, 0.2, 0.8).into()),
        block2: materials.add(Color::rgb(0.7, 0.7, 0.2).into()),
        block3: materials.add(Color::rgb(0.6, 0.6, 0.6).into()),
        block4: materials.add(Color::rgb(0.5, 0.2, 0.5).into()),
        block5: materials.add(Color::rgb(0.4, 0.4, 0.2).into()),
        block6: materials.add(Color::rgb(0.3, 0.3, 0.3).into()),
        block7: materials.add(Color::rgb(0.2, 0.2, 0.2).into()),
        block8: materials.add(Color::rgb(0.1, 0.1, 0.1).into()),
        block9: materials.add(Color::rgb(0.0, 0.0, 0.0).into()),
    });
}

fn block_spawn(
    mut cmd: Commands,
    mut ev: EventReader<SwipeEvent>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    q: Query<&Position, With<Block>>,
) {
    if ev.iter().next().is_none() {
        return;
    }
    let all_choices = (0u32..HEIGHT)
        .cartesian_product(0u32..WIDTH)
        .collect::<HashSet<_>>();
    let ex_choices = q.iter().map(|pos| (pos.x, pos.y)).collect::<HashSet<_>>();
    let rest = all_choices.difference(&ex_choices).collect::<Vec<_>>();

    let (x, y) = rest[rand::random::<usize>() % rest.len()];

    let v = rand::random::<u32>() % 1;
    cmd.spawn_bundle(SpriteBundle {
        material: materials.add(Color::rgb(0.9, 0.9, 0.9).into()),
        sprite: Sprite::new(Vec2::new(10.0, 10.0)),
        ..Default::default()
    })
    .insert(Block {
        value: v,
        merged: false,
    })
    .insert(Size { x: 0.8, y: 0.8 })
    .insert(Position { x: *x, y: *y })
    .id();
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_event::<SwipeEvent>()
        .insert_resource(WindowDescriptor {
            title: "2048".to_string(),
            width: 500.0,
            height: 500.0,
            ..Default::default()
        })
        .add_system(block_swipe.system())
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.15)))
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(block_pos.system())
                .with_system(block_scale.system())
                .with_system(game_over.system())
        )
        .add_system(block_spawn.system())
        .add_startup_system(setup.system())
        .add_system(block_color.system())
        .add_system_set(SystemSet::new().with_run_criteria(FixedTimestep::step(1.0)))
        .add_system(input_events.system())
        .run();
}
