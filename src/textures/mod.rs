use crate::Texture;
pub mod BRAT;
pub mod BRICK_1A;
pub mod BRICK_2B;
pub mod BRICK_3D;
pub mod CONSOLE_1B;
pub mod ORANGE_TILE;
pub mod SLIME_1A;
pub mod TILE_1A;
pub mod TILE_2C;
pub mod WOOD_1C;

//divider line do not change or remove!!!

pub const BRAT_TEXTURE: Texture = Texture {
    name: "BRAT_TEXTURE",
    width: BRAT::BRAT_WIDTH,
    height: BRAT::BRAT_HEIGHT,
    data: &BRAT::BRAT_ARRAY,
};

pub const ORANGE_TILE_TEXTURE: Texture = Texture {
    name: "ORANGE_TILE_TEXTURE",
    width: ORANGE_TILE::ORANGE_TILE_WIDTH,
    height: ORANGE_TILE::ORANGE_TILE_HEIGHT,
    data: &ORANGE_TILE::ORANGE_TILE_ARRAY,
};

pub const BRICK_1A_TEXTURE: Texture = Texture {
    name: "BRICK_1A_TEXTURE",
    width: BRICK_1A::BRICK_1A_WIDTH,
    height: BRICK_1A::BRICK_1A_HEIGHT,
    data: &BRICK_1A::BRICK_1A_ARRAY,
};

pub const BRICK_2B_TEXTURE: Texture = Texture {
    name: "BRICK_2B_TEXTURE",
    width: BRICK_2B::BRICK_2B_WIDTH,
    height: BRICK_2B::BRICK_2B_HEIGHT,
    data: &BRICK_2B::BRICK_2B_ARRAY,
};

pub const BRICK_3D_TEXTURE: Texture = Texture {
    name: "BRICK_3D_TEXTURE",
    width: BRICK_3D::BRICK_3D_WIDTH,
    height: BRICK_3D::BRICK_3D_HEIGHT,
    data: &BRICK_3D::BRICK_3D_ARRAY,
};

pub const CONSOLE_1B_TEXTURE: Texture = Texture {
    name: "CONSOLE_1B_TEXTURE",
    width: CONSOLE_1B::CONSOLE_1B_WIDTH,
    height: CONSOLE_1B::CONSOLE_1B_HEIGHT,
    data: &CONSOLE_1B::CONSOLE_1B_ARRAY,
};

pub const SLIME_1A_TEXTURE: Texture = Texture {
    name: "SLIME_1A_TEXTURE",
    width: SLIME_1A::SLIME_1A_WIDTH,
    height: SLIME_1A::SLIME_1A_HEIGHT,
    data: &SLIME_1A::SLIME_1A_ARRAY,
};

pub const TILE_1A_TEXTURE: Texture = Texture {
    name: "TILE_1A_TEXTURE",
    width: TILE_1A::TILE_1A_WIDTH,
    height: TILE_1A::TILE_1A_HEIGHT,
    data: &TILE_1A::TILE_1A_ARRAY,
};

pub const TILE_2C_TEXTURE: Texture = Texture {
    name: "TILE_2C_TEXTURE",
    width: TILE_2C::TILE_2C_WIDTH,
    height: TILE_2C::TILE_2C_HEIGHT,
    data: &TILE_2C::TILE_2C_ARRAY,
};

pub const WOOD_1C_TEXTURE: Texture = Texture {
    name: "WOOD_1C_TEXTURE",
    width: WOOD_1C::WOOD_1C_WIDTH,
    height: WOOD_1C::WOOD_1C_HEIGHT,
    data: &WOOD_1C::WOOD_1C_ARRAY,
};

pub const TEXTURES: [Texture; 10] = [
    BRAT_TEXTURE,
    ORANGE_TILE_TEXTURE,
    BRICK_1A_TEXTURE,
    BRICK_2B_TEXTURE,
    BRICK_3D_TEXTURE,
    CONSOLE_1B_TEXTURE,
    SLIME_1A_TEXTURE,
    TILE_1A_TEXTURE,
    TILE_2C_TEXTURE,
    WOOD_1C_TEXTURE,
];
