

use super::{
    QuanCoord,
    MapProcAsGame,
    MapInfo,
    PMState,
};

use std::sync::{
    Arc,
    Mutex,
    mpsc::{
        sync_channel,
    },
};

fn create_map_mock() -> MapInfo {
    let map_string = "01010\n00000\n31014\n10110\n01110".to_string();
    // 01010 <- (4, 4)
    // 00000
    // 31014
    // 10110
    // 01110
    // ^
    // (0, 0)
    MapInfo::build_by_string(map_string)
}



#[test]
fn map_mock_valid_test() {
    let m = create_map_mock();
    assert_eq!(m.width, 5);
    assert_eq!(m.height, 5);
}

#[test]
fn field_access_test() {
    let map = create_map_mock();
    assert_eq!(map.access_by_coord_game_based_system(QuanCoord{x: 0, y: 0}), 0);
    assert_eq!(map.access_by_coord_game_based_system(QuanCoord{x: 1, y: 1}), 0);
    assert_eq!(map.access_by_coord_game_based_system(QuanCoord{x: 1, y: 4}), 1);
    assert_eq!(map.access_by_coord_game_based_system(QuanCoord{x: 3, y: 1}), 1);
    assert_eq!(map.access_by_coord_game_based_system(QuanCoord{x: 3, y: 1}), 1);
    assert_eq!(map.access_by_coord_game_based_system(QuanCoord{x: 5, y: 2}), 3);
    assert_eq!(map.access_by_coord_game_based_system(QuanCoord{x: 6, y: 2}), 1);
    assert_eq!(map.access_by_coord_game_based_system(QuanCoord{x: 7, y: 2}), 0);
    assert_eq!(map.access_by_coord_game_based_system(QuanCoord{x: 8, y: 2}), 1);
}

#[test]
fn unique_field_element_search_test() {
    let map = create_map_mock();
    assert_eq!(*map.unique_points.get(&3).unwrap(), QuanCoord{x: 0, y: 2});
    assert_eq!(*map.unique_points.get(&4).unwrap(), QuanCoord{x: 4, y: 2});
}

#[test]
fn map_file_import_test() {
    let map = MapInfo::build_by_filename("C:\\Users\\Rock0x3FA\\OtobokeServer\\maps\\default_map".to_string());
    println!("({}, {})", map.width, map.height);
    //assert_eq!(0, 1);
}

#[test]
fn search_movable_point_test() {
    let map = create_map_mock();
    assert!(vec_group_eq(&map.get_can_move_on(QuanCoord{x: 2, y: 2}), &vec![QuanCoord{x: 2, y: 3}]));
    assert!(vec_group_eq(&map.get_can_move_on(QuanCoord{x: 2, y: 3}), &vec![QuanCoord{x: 1, y: 3}, QuanCoord{x: 2, y: 2}, QuanCoord{x: 2, y: 4}, QuanCoord{x: 3, y: 3}]));
    assert!(vec_group_eq(&map.get_can_move_on(QuanCoord{x: 4, y: 2}), &vec![QuanCoord{x: 0, y: 2}, QuanCoord{x: 4, y: 1}, QuanCoord{x: 4, y: 3}]));
    assert!(vec_group_eq(&map.get_can_move_on(QuanCoord{x: 0, y: 3}), &vec![QuanCoord{x: 0, y: 4}, QuanCoord{x: 0, y: 2}, QuanCoord{x: 1, y: 3}, QuanCoord{x: 4, y: 3}]));
    assert!(vec_group_eq(&map.get_can_move_on(QuanCoord{x: 4, y: 4}), &vec![QuanCoord{x: 0, y: 4}, QuanCoord{x: 4, y: 0}, QuanCoord{x: 4, y: 3}]));
}
// 01010 <- (4, 4)
// 00000
// 31014
// 10110
// 01110
// ^
// (0, 0)


#[test]
fn vec_group_eq_test() {
    let a = vec![1, 2, 3, 4];
    let b = vec![4, 3, 2, 1];
    let c = vec![1, 2, 3];
    let d = vec![1, 2, 3, 5];

    assert_eq!(vec_group_eq(&a, &a), true);
    assert_eq!(vec_group_eq(&a, &b), true);
    assert_eq!(vec_group_eq(&a, &c), false);
    assert_eq!(vec_group_eq(&a, &d), false);
    assert_eq!(vec_group_eq(&b, &a), true);
    assert_eq!(vec_group_eq(&b, &a), true);
    assert_eq!(vec_group_eq(&b, &b), true);
    assert_eq!(vec_group_eq(&b, &c), false);
    assert_eq!(vec_group_eq(&b, &d), false);
}

fn vec_group_eq<T: PartialEq>(v: &Vec<T>, w: &Vec<T>) -> bool {
    if v.len() != w.len() {
        return false;
    }
    !v.iter().any(|e| !w.iter().any(|x| *x == *e))
}

fn create_map_proc_as_game_mock() -> MapProcAsGame {
    MapProcAsGame {
        map: create_map_mock(), 
        players: vec![],
        pacman: QuanCoord::default(),
        pm_target: 0,
        pm_inferpoints: create_map_mock().get_inferpoints(),
        pm_state: Arc::new(Mutex::new(PMState::Normal)),
        pm_prev_place: QuanCoord::default(),
        comn_prov: None,
        snd: (sync_channel(0)).0,
    }
}
// 01010 <- (4, 4)
// 00000
// 31014
// 10110
// 01110
// ^
// (0, 0)

#[test]
fn routed_next_point_test() {
    let mut mock = create_map_proc_as_game_mock();
    mock.pacman = QuanCoord{x: 1, y: 3};
    mock.pm_prev_place = QuanCoord{x: 0, y: 3};
    assert_eq!(mock.routed_next_point(mock.map.get_can_move_on(mock.pacman)), QuanCoord{x: 2, y: 3});

    mock.pacman = QuanCoord{x: 4, y: 1};
    mock.pm_prev_place = QuanCoord{x: 4, y: 2};
    assert_eq!(mock.routed_next_point(mock.map.get_can_move_on(mock.pacman)), QuanCoord{x: 4, y: 0});
}

#[test]
fn move_to_non_teleport_point_test() {
    let mut mock = create_map_proc_as_game_mock();
    mock.pacman = QuanCoord{x: 2, y: 2};
    mock.move_to(QuanCoord{x: 2, y: 3}).unwrap();
    assert_eq!(mock.pacman, QuanCoord{x: 2, y: 3});
    assert_eq!(mock.pm_prev_place, QuanCoord{x: 2, y: 2});

    mock.pacman = QuanCoord{x: 2, y: 2};
    mock.move_to(QuanCoord{x: 2, y: 3}).unwrap();
    assert_eq!(mock.pacman, QuanCoord{x: 2, y: 3});
    assert_eq!(mock.pm_prev_place, QuanCoord{x: 2, y: 2});

    mock.pacman = QuanCoord{x: 1, y: 1};
    mock.pm_prev_place = QuanCoord{x: -1, y: -1};
    let result = mock.move_to(QuanCoord{x: 1, y: 2}); // (1, 2) is wall. so expected return is Err( (1, 2)).
    assert_eq!(result, Err(QuanCoord{x: 1, y: 2}));
    assert_eq!(mock.pacman, QuanCoord{x: 1, y: 1}); 
    assert_eq!(mock.pm_prev_place, QuanCoord{x: -1, y: -1}); // and. not change.
}

#[test]
fn move_to_teleport_point_test() {
    let mut mock = create_map_proc_as_game_mock();
    mock.pacman = QuanCoord{x: 0, y: 1};
    mock.move_to(QuanCoord{x: 0, y: 2}).unwrap(); // go to '3' teleport point
    assert_eq!(mock.pacman, QuanCoord{x: 4, y: 2});
    assert_eq!(mock.pm_prev_place, QuanCoord{x: 0, y: 2});

    mock.pacman = QuanCoord{x: 4, y: 1};
    mock.move_to(QuanCoord{x: 4, y: 2}).unwrap(); // go to '3' teleport point
    assert_eq!(mock.pacman, QuanCoord{x: 0, y: 2});
    assert_eq!(mock.pm_prev_place, QuanCoord{x: 4, y: 2});
}

// 01010 <- (4, 4)
// 00000
// 31014
// 10110
// 01110
// ^
// (0, 0)


