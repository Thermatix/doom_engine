

use super::*;

/// Draw the players location
pub fn draw_player<'m, 'c, M: Manager + FlagsData + ColoursStore>(canvas: &'c mut Canvas<Window>,  context: &'c Context, manager: &'m M ) {    
    let player = &context.player;   
    let map = &context.current_map;

    let scaled_pos = helpers::scale_xy(
        player.x,
        player.y,
        &map.map_bounds(),
        (manager.screen_width(), manager.screen_height()),
        30,
    );

    canvas.filled_circle(scaled_pos.0, scaled_pos.1, 5, Color::GREEN).unwrap();

}

pub fn draw_map_bsp<'m, 'c, M: Manager + FlagsData + ColoursStore>(canvas: &'c mut Canvas<Window>,  context: &'c Context, manager: &'m M ) {
    if manager.meta().get("don't_draw_bsp").is_some_and(|v| *v) { return };

    let map = &context.current_map;
    let player = &context.player;
    //let bsp = &context.bsp;

    let bounds = (manager.screen_width(), manager.screen_height());
    let boarder: i16 = 30;

    for node in map.traverse_bsp((player.x, player.y)) {
        let ((fx, fy), (fw, fh)) = get_bounding_box(&node.front_bbox, &map,bounds, boarder);
        let ((bx, by), (bw, bh)) = get_bounding_box(&node.back_bbox, &map,bounds, boarder);

        let (p1x, p1y) = helpers::scale_xy(node.x_partion, node.y_partion, map.map_bounds(), bounds, boarder);
        let (p2x, p2y) = helpers::scale_xy(node.dx_partion + node.x_partion, node.dy_partion + node.y_partion, map.map_bounds(), bounds, boarder);

        canvas.rectangle(fx, fy, fw, fh, Color::GREEN).unwrap();
        canvas.rectangle(bx, by, bw, bh, Color::RED).unwrap();

        canvas.thick_line(p1x, p1y, p2x, p2y,3, Color::BLUE).unwrap();
    }
    manager.mut_meta().insert("don't_draw_bsp".to_string(), true);
}

pub fn get_bounding_box(bbox: &wad::BoundingBox, map: &wad::Map, bounds: (i16, i16), boarder: i16) -> ((i16, i16), (i16, i16)) {
    (
        helpers::scale_xy(bbox.x, bbox.y,  map.map_bounds(), bounds, boarder),
        helpers::scale_xy(bbox.w, bbox.h,  map.map_bounds(), bounds, boarder)
    )
}

pub fn  draw_map_vertexes<'m, 'c, M: Manager + FlagsData + ColoursStore>(canvas: &'c mut Canvas<Window>,  context: &'c Context, manager: &'m M ) {
    if manager.meta().get("don't_draw_lines").is_some_and(|v| *v ) { return };
    let numbers = helpers::numbers();
    
    let segment_id = *numbers.get("segment_id").unwrap();

    let map = &context.current_map;   

    let seg = &map.segments[segment_id];

 

    let points = helpers::scale_map_points(
        map.map_points(),
        map.map_bounds(),
        (manager.screen_width(), manager.screen_height()),
        30
    );


    let p1 = points[seg.start_vertext_id as usize];
    let p2 = points[seg.end_verext_id as usize];
    
    canvas.filled_circle(p1.0, p1.1 , 2, Color::YELLOW).unwrap();
    canvas.filled_circle(p2.0, p2.1 , 2, Color::YELLOW).unwrap();

}

pub fn draw_map_line_defs<'m, 'c, M: Manager + FlagsData + ColoursStore>(canvas: &'c mut Canvas<Window>,  context: &'c Context, manager: &'m M ) {
    if manager.meta().get("don't_draw_vertexes").is_some_and(|v| *v ) { return };
    
    let map = &context.current_map;

    let points = helpers::scale_map_points(
        map.map_points(),
        map.map_bounds(),
        (manager.screen_width(), manager.screen_height()),
        30
    );

    for (p1, p2) in map.line_defs_to_vertexes(Some(&points)) {
        helpers::draw_line(canvas, p1, p2, Color::GREY);
    }
    manager.mut_meta().insert("don't_draw_vertexes".to_string(), true);
}


pub fn draw_map_lines_bsp<'m, 'c, M: Manager + FlagsData + ColoursStore>(canvas: &'c mut Canvas<Window>,  context: &'c Context, manager: &'m M ) {
    if manager.meta().get("don't_draw_lines").is_some_and(|v| *v ) { return };
    let mut numbers = helpers::numbers();
    
    let mut sub_sector_id = *numbers.entry("sub_sector_id".to_string()).or_insert(0);
    let mut segment_id = *numbers.entry("segment_id".to_string()).or_insert(0);
    

    let map = &context.current_map;
    let player = &context.player;
    
    let max_bounds = (manager.screen_width(), manager.screen_height());
    let boarder: i16 = 30;

    let points = helpers::scale_map_points(
        map.map_points(),
        map.map_bounds(),
        max_bounds,
        boarder
    );


    let nodes: Vec<wad::Node> = map.traverse_bsp((player.x, player.y)).collect();

    let segs_by_sub_sector_id = map.segs_from_nodes(&nodes, (player.x, player.y));


    let segs_to_draw = &segs_by_sub_sector_id[sub_sector_id];

    
    helpers::draw_seg(&canvas, &segs_to_draw.segments[segment_id], segs_to_draw.sub_sector_id, &points, &mut manager.mut_colours());


    segment_id += 1;

    if segment_id == segs_to_draw.segments.len() {
        sub_sector_id += 1;
        if sub_sector_id == segs_by_sub_sector_id.len() {
            manager.mut_meta().insert("don't_draw_lines".to_string(), true);
        } else {
            numbers.insert("segment_id".to_string(), 0);
            numbers.insert("sub_sector_id".to_string(), sub_sector_id);
        }
        
    } else {
        numbers.insert("segment_id".to_string(), segment_id);
    }

}
