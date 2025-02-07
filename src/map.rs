use std::collections::{hash_map::Entry, HashMap};

use egui::{Mesh, Painter, Pos2, Response, Sense, Ui, Widget};

use crate::{
    mercator::{screen_to_position, PositionExt, TileId},
    Position, Tiles, Zoom,
};

/// Slippy map widget.
pub struct Map<'a, 'b> {
    tiles: Option<&'b mut Tiles>,
    memory: &'a mut MapMemory,
    my_position: Position,
}

impl<'a, 'b> Map<'a, 'b> {
    pub fn new(
        tiles: Option<&'b mut Tiles>,
        memory: &'a mut MapMemory,
        my_position: Position,
    ) -> Self {
        Self {
            tiles,
            memory,
            my_position,
        }
    }
}

impl Widget for Map<'_, '_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::drag());

        self.memory
            .center_mode
            .screen_drag(&response, self.my_position, *self.memory.zoom);

        let map_center = self.memory.center_mode.position(self.my_position);
        let painter = ui.painter().with_clip_rect(rect);

        if let Some(tiles) = self.tiles {
            let mut meshes = Default::default();
            draw_tiles(
                &painter,
                map_center.tile_id(*self.memory.zoom),
                map_center.project(*self.memory.zoom),
                tiles,
                ui,
                &mut meshes,
            );

            for (_, shape) in meshes {
                painter.add(shape);
            }
        }

        response
    }
}

#[derive(Clone, PartialEq)]
pub enum MapCenterMode {
    MyPosition,
    Exact(Position),
}

impl MapCenterMode {
    fn screen_drag(&mut self, response: &Response, my_position: Position, zoom: u8) {
        if response.dragged_by(egui::PointerButton::Primary) {
            *self = match *self {
                // This makes it "detach" from the "my position".
                MapCenterMode::MyPosition => MapCenterMode::Exact(my_position),

                // Just shift already "detached" position.
                MapCenterMode::Exact(position) => MapCenterMode::Exact(screen_to_position(
                    position.project(zoom) - response.drag_delta(),
                    zoom,
                )),
            };
        }
    }

    pub fn position(&self, my_position: Position) -> Position {
        match self {
            MapCenterMode::MyPosition => my_position,
            MapCenterMode::Exact(position) => *position,
        }
    }
}

/// State of the map widget which must persist between frames.
pub struct MapMemory {
    pub center_mode: MapCenterMode,
    pub zoom: Zoom,
}

impl Default for MapMemory {
    fn default() -> Self {
        Self {
            center_mode: MapCenterMode::MyPosition,
            zoom: Default::default(),
        }
    }
}

fn draw_tiles(
    painter: &Painter,
    tile_id: TileId,
    map_center_projected_position: Pos2,
    tiles: &mut Tiles,
    ui: &mut Ui,
    meshes: &mut HashMap<TileId, Mesh>,
) {
    let tile_projected = tile_id.project();
    let tile_screen_position = painter.clip_rect().center().to_vec2() + tile_projected.to_vec2()
        - map_center_projected_position.to_vec2();

    let image = if let Some(image) = tiles.at(tile_id) {
        image
    } else {
        return;
    };

    if painter
        .clip_rect()
        .intersects(image.rect(tile_screen_position))
    {
        if let Entry::Vacant(vacant) = meshes.entry(tile_id) {
            vacant.insert(image.mesh(tile_screen_position, ui.ctx()));

            for coordinates in [
                tile_id.north(),
                tile_id.east(),
                tile_id.south(),
                tile_id.west(),
            ] {
                draw_tiles(
                    painter,
                    coordinates,
                    map_center_projected_position,
                    tiles,
                    ui,
                    meshes,
                );
            }
        }
    }
}
