use crate::tile::Tile;
use byte_pair_encoding::BytePairEncoding;
use bytemuck::NoUninit;
use huffman_derive::huffman_derive;
use lazy_static::lazy_static;
use quickcheck::{Arbitrary, Gen};
use std::fmt::{Display, Formatter};
use PublicTile::*;

#[huffman_derive(
    Hidden => 40,
    Flag => 10,
    Exploded => 5,
    Adjacent0 => 25,
    Adjacent1 => 20,
    Adjacent2 => 12,
    Adjacent3 => 3,
    Adjacent4 => 0.5,
    Adjacent5 => 0.1,
    Adjacent6 => 0.04,
    Adjacent7 => 0.001,
    Adjacent8 => 0.0001,
    Newline => 15
)]
#[derive(Eq, PartialEq, Debug, Copy, Clone, Hash)]
#[derive(NoUninit)]
#[repr(u8)]
pub enum PublicTile {
    Hidden = 0,
    Flag = Tile::empty().with_flag().0,
    Exploded = Tile::empty().with_revealed().with_mine().0,
    Adjacent0 = Tile::empty().with_revealed().0,
    Adjacent1 = Tile::empty().with_revealed().0 + 1,
    Adjacent2 = Tile::empty().with_revealed().0 + 2,
    Adjacent3 = Tile::empty().with_revealed().0 + 3,
    Adjacent4 = Tile::empty().with_revealed().0 + 4,
    Adjacent5 = Tile::empty().with_revealed().0 + 5,
    Adjacent6 = Tile::empty().with_revealed().0 + 6,
    Adjacent7 = Tile::empty().with_revealed().0 + 7,
    Adjacent8 = Tile::empty().with_revealed().0 + 8,
    Newline = u8::MAX,
}

impl PublicTile {
    pub fn from_compressed_bytes(bytes: Vec<u8>) -> Vec<Self> {
        // Self::from_huffman_bytes(bytes)
        BPE.decode(&bytes[..])
            .iter().map(|&byte| PublicTile::from(byte))
            .collect()
    }
    
    pub fn compress_tiles(public_tiles: &[PublicTile]) -> Vec<u8> {
        // let mut bw = BitWriter::new();
        // for tile in public_tiles {
        //     tile.encode(&mut bw);
        // }
        // bw.to_bytes()
        BPE.encode(bytemuck::cast_slice(public_tiles))
    }
}

impl From<u8> for PublicTile {
    fn from(value: u8) -> Self {
        if value == 255 {
            Newline
        } else {
            Tile(value).into()
        }
    }
}

impl From<&Tile> for PublicTile {
    fn from(value: &Tile) -> Self {
        if value.is_revealed() {
            if value.is_mine() {
                Exploded
            } else {
                match value.adjacent() {
                    0 => Adjacent0,
                    1 => Adjacent1,
                    2 => Adjacent2,
                    3 => Adjacent3,
                    4 => Adjacent4,
                    5 => Adjacent5,
                    6 => Adjacent6,
                    7 => Adjacent7,
                    8 => Adjacent8,
                    _ => panic!("Uh oh what have we got here...")
                }
            }
        } else {
            if value.is_flag() {
                Flag
            } else {
                Hidden
            }
        }
    }
}

impl From<Tile> for PublicTile {
    fn from(value: Tile) -> Self {
        (&value).into()
    }
}


impl From<&PublicTile> for Tile {
    fn from(value: &PublicTile) -> Self {
        Tile(value.clone() as u8)
    }
}

impl From<PublicTile> for Tile {
    fn from(value: PublicTile) -> Self {
        (&value).into()
    }
}

impl Arbitrary for PublicTile {
    fn arbitrary(g: &mut Gen) -> Self {
        let tile = Tile::arbitrary(g);
        tile.into()
    }
}

impl Display for PublicTile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let tile = Tile::from(self);
        write!(f, "{tile}")
    }
}

lazy_static! {
    static ref BPE: BytePairEncoding = BytePairEncoding::from_replacements(vec![
        (1, (0, 0)),
        (2, (1, 1)),
        (3, (2, 2)),
        (4, (3, 3)),
        (5, (4, 4)),
        (6, (5, 5)),
        (7, (64, 64)),
        (8, (65, 65)),
        (9, (3, 2)),
        (10, (6, 6)),
        (11, (1, 0)),
        (12, (65, 7)),
        (13, (64, 65)),
        (14, (7, 7)),
        (15, (9, 0)),
        (16, (5, 4)),
        (17, (3, 11)),
        (18, (10, 6)),
        (19, (3, 1)),
        (20, (65, 66)),
        (21, (9, 1)),
        (22, (9, 11)),
        (23, (3, 0)),
        (24, (64, 8)),
        (25, (66, 66)),
        (26, (66, 8)),
        (27, (65, 13)),
        (28, (2, 11)),
        (29, (12, 7)),
        (30, (2, 1)),
        (31, (2, 0)),
        (33, (64, 66)),
        (34, (12, 65)),
        (35, (8, 8)),
        (36, (7, 65)),
        (37, (22, 65)),
        (38, (8, 66)),
        (39, (65, 24)),
        (40, (6, 4)),
        (41, (6, 5)),
        (42, (12, 13)),
        (43, (6, 16)),
        (44, (10, 4)),
        (45, (12, 14)),
        (46, (10, 5)),
        (47, (10, 16)),
        (48, (18, 4)),
        (49, (8, 65)),
        (50, (7, 13)),
        (51, (65, 64)),
        (52, (18, 5)),
        (53, (18, 16)),
        (54, (15, 66)),
        (55, (9, 66)),
        (56, (0, 66)),
        (57, (17, 66)),
        (58, (22, 66)),
        (59, (19, 66)),
        (60, (15, 27)),
        (61, (9, 12)),
        (62, (66, 65)),
        (63, (23, 66)),
        (73, (29, 65)),
        (74, (15, 65)),
        (75, (14, 65)),
        (76, (21, 51)),
        (77, (13, 66)),
        (78, (9, 34)),
        (79, (13, 21)),
        (81, (3, 66)),
        (82, (67, 66)),
        (83, (25, 66)),
        (84, (37, 37)),
        (85, (8, 20)),
        (86, (12, 24)),
        (87, (28, 66)),
        (88, (7, 66)),
        (89, (17, 42)),
        (90, (67, 8)),
        (91, (30, 66)),
        (92, (7, 24)),
        (93, (15, 12)),
        (94, (36, 15)),
        (95, (27, 66)),
        (96, (20, 66)),
        (97, (29, 13)),
        (98, (31, 66)),
        (99, (7, 8)),
        (100, (14, 13)),
        (101, (0, 65)),
        (102, (1, 66)),
        (103, (12, 8)),
        (104, (21, 66)),
        (105, (2, 66)),
        (106, (11, 66)),
        (107, (7, 64)),
        (108, (26, 65)),
        (109, (19, 29)),
        (110, (7, 20)),
        (111, (12, 66)),
        (112, (21, 62)),
        (113, (17, 26)),
        (114, (12, 33)),
        (115, (65, 33)),
        (116, (7, 33)),
        (117, (17, 8)),
        (118, (15, 26)),
        (119, (50, 9)),
        (120, (12, 20)),
        (121, (21, 65)),
        (122, (14, 14)),
        (123, (19, 73)),
        (124, (61, 64)),
        (125, (39, 66)),
        (126, (0, 67)),
        (127, (12, 64)),
        (128, (25, 8)),
        (129, (15, 67)),
        (130, (9, 67)),
        (131, (24, 66)),
        (132, (21, 8)),
        (133, (19, 8)),
        (134, (15, 49)),
        (135, (0, 26)),
        (136, (19, 26)),
        (137, (3, 65)),
        (138, (3, 45)),
        (139, (21, 79)),
        (140, (17, 67)),
        (141, (65, 67)),
        (142, (31, 65)),
        (143, (19, 65)),
        (144, (17, 65)),
        (145, (2, 65)),
        (146, (23, 26)),
        (147, (25, 67)),
        (148, (80, 65)),
        (149, (11, 65)),
        (150, (23, 65)),
        (151, (1, 65)),
        (152, (23, 8)),
        (153, (28, 65)),
        (154, (30, 65)),
        (155, (17, 29)),
        (156, (9, 65)),
        (157, (0, 27)),
        (158, (22, 67)),
        (159, (9, 8)),
        (160, (74, 33)),
        (161, (15, 94)),
        (162, (14, 36)),
        (163, (19, 67)),
        (164, (54, 13)),
        (165, (20, 8)),
        (166, (8, 67)),
        (167, (104, 64)),
        (168, (61, 66)),
        (169, (15, 38)),
        (170, (50, 66)),
        (171, (23, 67)),
        (172, (3, 26)),
        (173, (37, 58)),
        (174, (76, 76)),
        (175, (55, 36)),
        (176, (23, 97)),
        (177, (25, 65)),
        (178, (14, 66)),
        (179, (28, 45)),
        (180, (54, 20)),
        (181, (3, 8)),
        (182, (17, 75)),
        (183, (20, 67)),
        (184, (35, 65)),
        (185, (3, 67)),
        (186, (9, 26)),
        (187, (7, 38)),
        (188, (9, 119)),
        (189, (23, 29)),
        (190, (55, 27)),
        (191, (30, 67)),
        (192, (15, 39)),
        (193, (42, 66)),
        (194, (54, 7)),
        (195, (109, 64)),
        (196, (21, 67)),
        (197, (54, 51)),
        (198, (19, 35)),
        (199, (28, 26)),
        (200, (28, 67)),
        (201, (80, 66)),
        (202, (1, 67)),
        (203, (13, 0)),
        (204, (12, 38)),
        (205, (27, 0)),
        (206, (57, 50)),
        (207, (28, 8)),
        (208, (58, 58)),
        (209, (78, 9)),
        (210, (2, 67)),
        (211, (24, 8)),
        (212, (31, 67)),
        (213, (17, 114)),
        (214, (14, 64)),
        (215, (30, 26)),
        (216, (24, 65)),
        (217, (39, 65)),
        (218, (19, 100)),
        (219, (57, 34)),
        (220, (33, 21)),
        (221, (14, 33)),
        (222, (60, 60)),
        (223, (74, 148)),
        (224, (11, 67)),
        (225, (23, 35)),
        (226, (14, 50)),
        (227, (14, 20)),
        (228, (9, 27)),
        (229, (30, 45)),
        (230, (56, 20)),
        (231, (30, 8)),
        (232, (25, 25)),
        (233, (89, 17)),
        (234, (55, 107)),
        (235, (29, 66)),
        (236, (67, 67)),
        (237, (138, 65)),
        (238, (1, 26)),
        (239, (29, 64)),
        (240, (93, 93)),
        (241, (2, 26)),
        (242, (20, 21)),
        (243, (31, 26)),
        (244, (67, 17)),
        (245, (11, 26)),
        (246, (182, 17)),
        (247, (15, 88)),
        (248, (12, 49)),
        (249, (14, 8)),
        (250, (14, 7)),
        (251, (59, 75)),
        (252, (29, 8)),
        (253, (45, 65)),
        (254, (3, 35)),
    ]);
}