use amethyst::core::math::{dot, zero, Isometry2, Vector2};
use ncollide2d::{
    pipeline::{
        narrow_phase::{ContactEvent, ProximityEvent},
        object::{CollisionGroups, CollisionObjectSlabHandle, GeometricQueryType},
        world::CollisionWorld,
    },
    query::Proximity,
    shape::{Ball, Cuboid, Plane, ShapeHandle},
};
use std::cell::Cell;

#[derive(Clone)]
struct CollisionObjectData {
    pub name: &'static str,
    pub velocity: Option<Cell<Vector2<f32>>>,
}

impl CollisionObjectData {
    pub fn new(name: &'static str, velocity: Option<Vector2<f32>>) -> CollisionObjectData {
        let init_velocity;
        if let Some(velocity) = velocity {
            init_velocity = Some(Cell::new(velocity))
        } else {
            init_velocity = None
        }

        CollisionObjectData {
            name: name,
            velocity: init_velocity,
        }
    }
}

fn handle_contact_event(
    world: &CollisionWorld<f32, CollisionObjectData>,
    event: &ContactEvent<CollisionObjectSlabHandle>,
) {
    if let &ContactEvent::Started(collider1, collider2) = event {
        // NOTE: real-life applications would avoid this systematic allocation.
        let (_, _, _, pair) = world.contact_pair(collider1, collider2, true).unwrap();
        let mut collector: Vec<_> = pair.contacts().collect();

        let co1 = world.collision_object(collider1).unwrap();
        let co2 = world.collision_object(collider2).unwrap();

        // The ball is the one with a non-None velocity.
        if let Some(ref vel) = co1.data().velocity {
            let normal = collector[0].contact.normal;
            vel.set(vel.get() - 2.0 * dot(&vel.get(), &normal) * *normal);
        }
        if let Some(ref vel) = co2.data().velocity {
            let normal = -collector[0].contact.normal;
            vel.set(vel.get() - 2.0 * dot(&vel.get(), &normal) * *normal);
        }
    }
}

fn handle_proximity_event(
    world: &CollisionWorld<f32, CollisionObjectData>,
    event: &ProximityEvent<CollisionObjectSlabHandle>,
) {
    // The collision object with a None velocity is the coloured area.
    let area_name;
    let co1 = world.collision_object(event.collider1).unwrap();
    let co2 = world.collision_object(event.collider2).unwrap();

    if co1.data().velocity.is_none() {
        area_name = co1.data().name;
    } else {
        area_name = co2.data().name;
    }

    if event.new_status == Proximity::Intersecting {
        println!("The ball enters the {} area.", area_name);
    } else if event.new_status == Proximity::Disjoint {
        println!("The ball leaves the {} area.", area_name);
    }
}

fn main() {
    let plane_left = ShapeHandle::new(Plane::new(Vector2::x_axis()));
    let plane_bottom = ShapeHandle::new(Plane::new(Vector2::y_axis()));
    let plane_right = ShapeHandle::new(Plane::new(-Vector2::x_axis()));
    let plane_top = ShapeHandle::new(Plane::new(-Vector2::y_axis()));

    let rect = ShapeHandle::new(Cuboid::new(Vector2::new(4.0f32, 4.0)));
    // Ball shape.
    let ball = ShapeHandle::new(Ball::new(0.5f32));

    // position of the planes (to be replaced by transform in amethyst.

    // Positions of the planes.
    let planes_pos = [
        Isometry2::new(Vector2::new(-10.0, 0.0), zero()),
        Isometry2::new(Vector2::new(0.0, -10.0), zero()),
        Isometry2::new(Vector2::new(10.0, 0.0), zero()),
        Isometry2::new(Vector2::new(0.0, 10.0), zero()),
    ];

    // Position of the rectangles.
    let rects_pos = [
        Isometry2::new(Vector2::new(-5.0, 5.0), zero()),
        Isometry2::new(Vector2::new(5.0, 5.0), zero()),
        Isometry2::new(Vector2::new(5.0, -5.0), zero()),
        Isometry2::new(Vector2::new(-5.0, -5.0), zero()),
    ];

    // Position of the ball.
    let ball_pos = Isometry2::new(Vector2::new(5.0, 5.0), zero());

    // The ball is part of group 1 and can interact with everything.
    let mut ball_groups = CollisionGroups::new();
    ball_groups.set_membership(&[1]);

    // All the other objects are part of the group 2 and interact only with the ball (but not with
    // each other).
    let mut others_groups = CollisionGroups::new();
    others_groups.set_membership(&[2]);
    others_groups.set_whitelist(&[1]);

    let plane_data = CollisionObjectData::new("ground", None);
    let rect_data_purple = CollisionObjectData::new("purple", None);
    let rect_data_blue = CollisionObjectData::new("blue", None);
    let rect_data_green = CollisionObjectData::new("green", None);
    let rect_data_yellow = CollisionObjectData::new("yellow", None);
    let ball_data = CollisionObjectData::new("ball", Some(Vector2::new(10.0, 5.0)));

    /*
     * Setup the world.
     */
    // Collision world 0.02 optimization margin and small object identifiers.
    let mut world = CollisionWorld::new(0.02);

    // Add the planes to the world.
    let contacts_query = GeometricQueryType::Contacts(0.0, 0.0);
    let proximity_query = GeometricQueryType::Proximity(0.0);

    world.add(
        planes_pos[0],
        plane_left,
        others_groups,
        contacts_query,
        plane_data.clone(),
    );
    world.add(
        planes_pos[1],
        plane_bottom,
        others_groups,
        contacts_query,
        plane_data.clone(),
    );
    world.add(
        planes_pos[2],
        plane_right,
        others_groups,
        contacts_query,
        plane_data.clone(),
    );
    world.add(
        planes_pos[3],
        plane_top,
        others_groups,
        contacts_query,
        plane_data.clone(),
    );

    // Add the colored rectangles to the world.
    world.add(
        rects_pos[0],
        rect.clone(),
        others_groups,
        proximity_query,
        rect_data_purple,
    );
    world.add(
        rects_pos[1],
        rect.clone(),
        others_groups,
        proximity_query,
        rect_data_blue,
    );
    world.add(
        rects_pos[2],
        rect.clone(),
        others_groups,
        proximity_query,
        rect_data_green,
    );
    world.add(
        rects_pos[3],
        rect.clone(),
        others_groups,
        proximity_query,
        rect_data_yellow,
    );

    // Add the ball to the world.
    let (ball_handle, _) = world.add(ball_pos, ball, ball_groups, contacts_query, ball_data);

    // Register our handlers.
    //world.register_proximity_handler("ProximityMessage", ProximityMessage);
    //world.register_contact_handler("VelocityBouncer", VelocityBouncer);

    /*
     * Run indefinitely.
     */
    let timestep = 0.016;

    loop {
        // Poll and handle events.
        for event in world.proximity_events() {
            handle_proximity_event(&world, event)
        }

        for event in world.contact_events() {
            handle_contact_event(&world, event)
        }

        // Integrate velocities and positions.
        let mut ball_pos;
        {
            // Integrate the velocities.
            let ball_object = world.get_mut(ball_handle).unwrap();
            let ball_velocity = ball_object.data().velocity.as_ref().unwrap();

            // Integrate the positions.
            ball_pos = ball_object.position().clone();

            ball_pos.append_translation_mut(&(timestep * ball_velocity.get()).into());
            ball_object.set_position(ball_pos);
        }

        world.update();
    }
}
