//! Script format for all XX-series Guilty Gears
use binrw::binread;
use binrw::helpers::until;
use serde::{Deserialize, Serialize};

#[binread]
#[derive(Clone, Serialize, Deserialize)]
pub struct GGXXPlayerScriptData {
    pub play_data: PlayData,
    #[br(count = 0xB0)]
    pub unk_data: Vec<u8>,
    #[br(
        parse_with = until(|action: &ScriptAction| matches!(action.instructions.last().unwrap(), ScriptInstruction::ScriptEnd { .. }))
    )]
    pub actions: Vec<ScriptAction>,
}

#[binread]
#[derive(Clone, Serialize, Deserialize)]
pub struct GGXXObjScriptData {
    #[br(
        parse_with = until(|action: &ScriptAction| matches!(action.instructions.last().unwrap(), ScriptInstruction::ScriptEnd { .. }))
    )]
    pub actions: Vec<ScriptAction>,
}

#[binread]
#[br(magic = b"\xE5")]
#[derive(Clone, Serialize, Deserialize)]
pub struct PlayData {
    pub unk: u8,
    pub fwalk_vel: i16,
    pub bwalk_vel: i16,
    pub fdash_init_vel: i16,
    pub bdash_x_vel: i16,
    pub bdash_y_vel: i16,
    pub bdash_gravity: i16,
    pub fjump_x_vel: i16,
    pub bjump_x_vel: i16,
    pub jump_y_vel: i16,
    pub jump_gravity: i16,
    pub fsuperjump_x_vel: i16,
    pub bsuperjump_x_vel: i16,
    pub superjump_y_vel: i16,
    pub superjump_gravity: i16,
    pub fdash_accel: i16,
    pub fdash_reduce: i16,
    pub init_homingjump_y_vel: i16,
    pub init_homingjump_x_vel: i16,
    pub init_homingjump_x_reduce: i16,
    pub init_homingjump_y_offset: i16,
    pub airdash_minimum_height: i16,
    pub fairdash_time: i16,
    pub bairdash_time: i16,
    pub stun_res: i16,
    pub defense: i16,
    pub guts: i16,
    pub critical: i16,
    pub weight: i16,
    pub airdash_count: i16,
    pub airjump_count: i16,
    pub fairdash_no_attack_time: i16,
    pub bairdash_no_attack_time: i16,
    pub fwalk_tension: i16,
    pub fjump_tension: i16,
    pub fdash_tension: i16,
    pub fairdash_tension: i16,
    pub guardbalance_defense: i16,
    pub guardbalance_tension: i16,
    pub instantblock_tension: i16,
}

#[binread]
#[repr(u8)]
#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum ScriptInstruction {
    #[br(magic(0u8))]
    CellBegin {
        duration: u8,
        cell_no: u16,
    },
    #[br(magic(1u8))]
    Unk1 {
        flag: u8,
        arg: u16,
    },
    #[br(magic(2u8))]
    MakeObj {
        flag: u8,
        arg: u16,
    },
    #[br(magic(3u8))]
    BackMotion {
        flag: u8,
        arg: u16,
    },
    #[br(magic(4u8))]
    RenewCollision {
        flag: u8,
        arg: u16,
    },
    #[br(magic(5u8))]
    Shade {
        r: u8,
        g: u8,
        b: u8,
    },
    #[br(magic(6u8))]
    SemiTrans {
        flag: u8,
        arg: u16,
    },
    #[br(magic(7u8))]
    Scale {
        flag: u8,
        stretch: i16,
    },
    #[br(magic(8u8))]
    Rotation {
        flag: u8,
        angle: i16,
    },
    #[br(magic(9u8))]
    UnkAttack {
        flag: u8,
        x: i16,
        y: i16,
        priority: u16,
    },
    #[br(magic(11u8))]
    StopFrame {
        #[br(temp)]
        #[serde(skip)]
        flag: u8,
        #[br(temp)]
        #[serde(skip)]
        arg: u16,
    },
    #[br(magic(13u8))]
    DoNotCheckAttack {
        #[br(temp)]
        #[serde(skip)]
        flag: u8,
        #[br(temp)]
        #[serde(skip)]
        arg: u16,
    },
    #[br(magic(14u8))]
    DoNotCheckDamage {
        #[br(temp)]
        #[serde(skip)]
        flag: u8,
        #[br(temp)]
        #[serde(skip)]
        arg: u16,
    },
    #[br(magic(15u8))]
    Reverse {
        flag: u8,
        arg: u16,
    },
    #[br(magic(16u8))]
    DrawNormal {
        flag: u8,
        arg: u16,
    },
    #[br(magic(17u8))]
    DrawReverse {
        flag: u8,
        arg: u16,
    },
    #[br(magic(18u8))]
    ChainCancel {
        flag: u8,
        arg: u16,
    },
    #[br(magic(20u8))]
    Unk20 {
        flag: u8,
        arg: u16,
    },
    #[br(magic(25u8))]
    InitInstance {
        #[br(temp)]
        #[serde(skip)]
        rsrv: u8,
        anime_no: u16,
        obj_no: u32,
        kind: u32,
        state_no: u16,
        is_check_col: u16,
    },
    #[br(magic(26u8))]
    DeleteChildInstance {
        obj_no: u8,
        flag: u8,
        act_no: u8,
    },
    #[br(magic(27u8))]
    InitRqSound {
        data: u8,
        flag: u8,
        random_factor: u8,
    },
    #[br(magic(28u8))]
    InitEnemyHitSeMode {
        data: u8,
        flag: u8,
        random_factor: u8,
    },
    #[br(magic(30u8))]
    EnemyGuardModeVoice {
        data: u8,
        flag: u8,
        random_factor: u8,
    },
    #[br(magic(31u8))]
    EnemyDamageModeVoice {
        data: u8,
        flag: u8,
        random_factor: u8,
    },
    #[br(magic(32u8))]
    AttackModeVoice {
        data: u8,
        flag: u8,
        random_factor: u8,
    },
    #[br(magic(33u8))]
    DownReturn {
        flag: u8,
        arg: u16,
    },
    #[br(magic(34u8))]
    DownJuuryoku {
        flag: u8,
        arg: u16,
    },
    #[br(magic(35u8))]
    DownX {
        flag: u8,
        arg: u16,
    },
    #[br(magic(36u8))]
    DownY {
        flag: u8,
        arg: u16,
    },
    #[br(magic(38u8))]
    DeleteIttai {
        flag: u8,
        arg: u16,
    },
    #[br(magic(39u8))]
    JumpCell {
        flag: u8,
        arg: u16,
    },
    #[br(magic(40u8))]
    PosZ {
        flag: u8,
        z: u16,
    },
    #[br(magic(45u8))]
    Unk45 {
        #[br(temp)]
        #[serde(skip)]
        rsrv: u8,
        arg: u16,
    },
    #[br(magic(47u8))]
    Unk47 {
        #[br(temp)]
        #[serde(skip)]
        rsrv: u8,
        arg: u16,
    },
    #[br(magic(48u8))]
    Unk48 {
        #[br(temp)]
        #[serde(skip)]
        rsrv: u8,
        arg: u16,
    },
    #[br(magic(49u8))]
    Goto {
        #[br(temp)]
        #[serde(skip)]
        flag: u8,
        #[br(temp)]
        #[serde(skip)]
        arg: u16,
        jump_target: u32,
    },
    #[br(magic(50u8))]
    Unk50 {
        flag: u8,
        arg: u16,
    },
    #[br(magic(52u8))]
    AddTension {
        flag: u8,
        arg: u16,
    },
    #[br(magic(53u8))]
    Unk53 {
        flag: u8,
        arg: u16,
        arg2: u16,
        arg3: u16,
    },
    #[br(magic(54u8))]
    Unk54 {
        flag: u8,
        arg: u16,
    },
    #[br(magic(55u8))]
    Unk55 {
        flag: u8,
        arg: u16,
    },
    #[br(magic(57u8))]
    Unk57 {
        flag: u8,
        arg: u16,
    },
    #[br(magic(58u8))]
    Unk58 {
        flag: u8,
        arg: u16,
    },
    #[br(magic(60u8))]
    Unk60 {
        flag: u8,
        arg: u16,
    },
    #[br(magic(61u8))]
    Unk61 {
        #[br(temp)]
        #[serde(skip)]
        flag: u8,
        #[br(temp)]
        #[serde(skip)]
        arg: u16,
    },
    #[br(magic(63u8))]
    Unk63 {
        flag: u8,
        arg: u16,
    },
    #[br(magic(64u8))]
    AttackLevelAndDamage {
        flag: u8,
        damage: u8,
        attack_level: u8,
    },
    #[br(magic(65u8))]
    XTransform {
        flag: u8,
        magnitude: i16,
    },
    #[br(magic(66u8))]
    YTransform {
        flag: u8,
        magnitude: i16,
    },
    #[br(magic(67u8))]
    SetGravity {
        flag: u8,
        magnitude: i16,
    },
    #[br(magic(68u8))]
    Unk68 {
        flag: u8,
        arg: u16,
    },
    #[br(magic(69u8))]
    Unk69 {
        flag: u8,
        arg: u16,
        arg2: u16,
        arg3: u16,
    },
    #[br(magic(71u8))]
    Unk71 {
        flag: u8,
        arg: u16,
        arg2: u32,
    },
    #[br(magic(72u8))]
    Unk72 {
        flag: u8,
        arg: u16,
    },
    #[br(magic(73u8))]
    Unk73 {
        flag: u8,
        arg: u16,
    },
    #[br(magic(74u8))]
    SetAttackStunVal {
        flag: u8,
        arg: u16,
    },
    #[br(magic(75u8))]
    Unk75 {
        flag: u8,
        arg: u16,
    },
    #[br(magic(76u8))]
    SetStance {
        flag: u8,
        arg: u16,
    },
    #[br(magic(77u8))]
    Unk77 {
        flag: u8,
        arg: u16,
    },
    #[br(magic(78u8))]
    Unk78 {
        flag: u8,
        arg: u16,
    },
    #[br(magic(79u8))]
    Unk79 {
        flag: u8,
        arg: u16,
    },
    #[br(magic(80u8))]
    Unk80 {
        flag: u8,
        arg: u16,
    },
    #[br(magic(81u8))]
    Unk81 {
        flag: u8,
        arg: u16,
    },
    #[br(magic(82u8))]
    Unk82 {
        flag: u8,
        arg: u16,
    },
    #[br(magic(84u8))]
    Pushback {
        flag: u8,
        magnitude: u16,
    },
    #[br(magic(85u8))]
    Stagger {
        flag: u8,
        arg: u16,
    },
    #[br(magic(86u8))]
    Unk86 {
        flag: u8,
        obj_id: u16,
        buffered_act: u16,
        act_id: u16,
    },
    #[br(magic(89u8))]
    Unk89 {
        flag: u8,
        arg: u16,
    },
    #[br(magic(92u8))]
    JumpInstall {
        flag: u8,
        arg: u16,
    },
    #[br(magic(94u8))]
    Unk94 {
        flag: u8,
        arg: u16,
        arg2: u32,
    },
    #[br(magic(95u8))]
    Unk95 {
        flag: u8,
        arg1: u16,
        arg2: u16,
        arg3: u16,
    },
    #[br(magic(96u8))]
    Unk96 {
        flag: u8,
        arg1: u16,
        arg2: u16,
        arg3: u16,
    },
    #[br(magic(99u8))]
    Unk99 {
        flag: u8,
        arg: u16,
    },
    #[br(magic(100u8))]
    Unk100 {
        flag: u8,
        arg1: u16,
        arg2: u32,
    },
    #[br(magic(101u8))]
    SetAttackProperties {
        flag: u8,
        arg: u16,
        unk1: u8,
        unk2: u8,
        unk3: u8,
        unk4: u8,
    },
    #[br(magic(132u8))]
    Unk132 {
        flag: u8,
        arg: u16,
    },
    #[br(magic(133u8))]
    Unk133 {
        flag: u8,
        arg: u16,
    },
    #[br(magic(253u8))]
    ScriptEnd {
        flag: u8,
        arg: u16,
    },
    #[br(magic(255u8))]
    ActionEnd {
        flag: u8,
        arg: u16,
    },
}

impl ScriptInstruction {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();

        match self {
            ScriptInstruction::CellBegin { duration, cell_no } => {
                buffer.push(0);
                buffer.push(*duration);
                buffer.append(&mut cell_no.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk1 { flag, arg } => {
                buffer.push(1);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::MakeObj { flag, arg } => {
                buffer.push(2);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::BackMotion { flag, arg } => {
                buffer.push(3);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::RenewCollision { flag, arg } => {
                buffer.push(4);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::Shade { r, g, b } => {
                buffer.push(5);
                buffer.push(*r);
                buffer.push(*g);
                buffer.push(*b);
            }
            ScriptInstruction::SemiTrans { flag, arg } => {
                buffer.push(6);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::Scale { flag, stretch } => {
                buffer.push(7);
                buffer.push(*flag);
                buffer.append(&mut stretch.to_le_bytes().to_vec());
            }
            ScriptInstruction::Rotation { flag, angle } => {
                buffer.push(8);
                buffer.push(*flag);
                buffer.append(&mut angle.to_le_bytes().to_vec());
            }
            ScriptInstruction::UnkAttack { flag: attack_level, x: damage, y: arg1, priority: arg2 } => {
                buffer.push(9);
                buffer.push(*attack_level);
                buffer.append(&mut damage.to_le_bytes().to_vec());
                buffer.append(&mut arg1.to_le_bytes().to_vec());
                buffer.append(&mut arg2.to_le_bytes().to_vec());
            }
            ScriptInstruction::StopFrame { .. } => {
                buffer.push(11);
                for _ in 0..3 {
                    buffer.push(0x0);
                }
            }
            ScriptInstruction::DoNotCheckAttack { .. } => {
                buffer.push(13);
                for _ in 0..3 {
                    buffer.push(0x0);
                }
            }
            ScriptInstruction::DoNotCheckDamage { .. } => {
                buffer.push(14);
                for _ in 0..3 {
                    buffer.push(0x0);
                }
            }
            ScriptInstruction::Reverse { flag, arg } => {
                buffer.push(15);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::DrawNormal { arg, flag } => {
                buffer.push(16);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::DrawReverse { arg, flag } => {
                buffer.push(17);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::ChainCancel { flag, arg } => {
                buffer.push(18);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk20 { flag, arg } => {
                buffer.push(20);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::InitInstance {
                anime_no, obj_no, kind, state_no, is_check_col }
            => {
                buffer.push(25);
                buffer.push(0);
                buffer.append(&mut anime_no.to_le_bytes().to_vec());
                buffer.append(&mut obj_no.to_le_bytes().to_vec());
                buffer.append(&mut kind.to_le_bytes().to_vec());
                buffer.append(&mut state_no.to_le_bytes().to_vec());
                buffer.append(&mut is_check_col.to_le_bytes().to_vec());
            }
            ScriptInstruction::DeleteChildInstance { obj_no, flag, act_no } => {
                buffer.push(26);
                buffer.push(*obj_no);
                buffer.push(*flag);
                buffer.push(*act_no);
            }
            ScriptInstruction::InitRqSound { data, flag, random_factor } => {
                buffer.push(27);
                buffer.push(*data);
                buffer.push(*flag);
                buffer.push(*random_factor);
            }
            ScriptInstruction::InitEnemyHitSeMode { data, flag, random_factor } => {
                buffer.push(28);
                buffer.push(*data);
                buffer.push(*flag);
                buffer.push(*random_factor);
            }
            ScriptInstruction::EnemyGuardModeVoice { data, flag, random_factor } => {
                buffer.push(30);
                buffer.push(*data);
                buffer.push(*flag);
                buffer.push(*random_factor);
            }
            ScriptInstruction::EnemyDamageModeVoice { data, flag, random_factor } => {
                buffer.push(31);
                buffer.push(*data);
                buffer.push(*flag);
                buffer.push(*random_factor);
            }
            ScriptInstruction::AttackModeVoice { data, flag, random_factor } => {
                buffer.push(32);
                buffer.push(*data);
                buffer.push(*flag);
                buffer.push(*random_factor);
            }
            ScriptInstruction::DownReturn { flag, arg } => {
                buffer.push(33);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::DownJuuryoku { flag, arg } => {
                buffer.push(34);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::DownX { flag, arg } => {
                buffer.push(35);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::DownY { flag, arg } => {
                buffer.push(36);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::DeleteIttai { flag, arg } => {
                buffer.push(38);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::JumpCell { flag, arg } => {
                buffer.push(39);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::PosZ { flag, z } => {
                buffer.push(40);
                buffer.push(*flag);
                buffer.append(&mut z.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk45 { arg } => {
                buffer.push(45);
                buffer.push(0);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk47 { arg } => {
                buffer.push(47);
                buffer.push(0);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk48 { arg } => {
                buffer.push(48);
                buffer.push(0);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::Goto { jump_target } => {
                buffer.push(49);
                for _ in 0..3 {
                    buffer.push(0x0);
                }
                buffer.append(&mut jump_target.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk50 { flag, arg } => {
                buffer.push(50);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::AddTension { flag, arg } => {
                buffer.push(52);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk53 { flag, arg, arg2, arg3 } => {
                buffer.push(53);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
                buffer.append(&mut arg2.to_le_bytes().to_vec());
                buffer.append(&mut arg3.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk54 { flag, arg } => {
                buffer.push(54);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk55 { flag, arg } => {
                buffer.push(55);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk57 { flag, arg } => {
                buffer.push(57);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk58 { flag, arg } => {
                buffer.push(58);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk60 { flag, arg } => {
                buffer.push(60);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk61 {} => {
                buffer.push(61);
                for _ in 0..3 {
                    buffer.push(0x0);
                }
            }
            ScriptInstruction::Unk63 { flag, arg } => {
                buffer.push(63);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::AttackLevelAndDamage { flag: attack_level, damage, attack_level: arg } => {
                buffer.push(64);
                buffer.push(*attack_level);
                buffer.push(*damage);
                buffer.push(*arg);
            }
            ScriptInstruction::XTransform { flag, magnitude } => {
                buffer.push(65);
                buffer.push(*flag);
                buffer.append(&mut magnitude.to_le_bytes().to_vec());
            }
            ScriptInstruction::YTransform { flag, magnitude } => {
                buffer.push(66);
                buffer.push(*flag);
                buffer.append(&mut magnitude.to_le_bytes().to_vec());
            }
            ScriptInstruction::SetGravity { flag, magnitude } => {
                buffer.push(67);
                buffer.push(*flag);
                buffer.append(&mut magnitude.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk68 { flag, arg } => {
                buffer.push(68);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk69  { flag, arg, arg2, arg3 } => {
                buffer.push(69);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
                buffer.append(&mut arg2.to_le_bytes().to_vec());
                buffer.append(&mut arg3.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk71 { flag, arg, arg2 } => {
                buffer.push(71);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
                buffer.append(&mut arg2.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk72 { flag, arg } => {
                buffer.push(72);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk73 { flag, arg } => {
                buffer.push(73);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::SetAttackStunVal { flag, arg } => {
                buffer.push(74);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk75 { flag, arg } => {
                buffer.push(75);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::SetStance { flag, arg } => {
                buffer.push(76);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk77 { flag, arg } => {
                buffer.push(77);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk78 { flag, arg } => {
                buffer.push(78);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk79 { flag, arg } => {
                buffer.push(79);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk80 { flag, arg } => {
                buffer.push(80);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk81 { flag, arg } => {
                buffer.push(81);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk82 { flag, arg } => {
                buffer.push(82);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::Pushback { flag, magnitude } => {
                buffer.push(84);
                buffer.push(*flag);
                buffer.append(&mut magnitude.to_le_bytes().to_vec());
            }
            ScriptInstruction::Stagger { flag, arg } => {
                buffer.push(85);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk86 { flag, obj_id, buffered_act, act_id, } => {
                buffer.push(86);
                buffer.push(*flag);
                buffer.append(&mut obj_id.to_le_bytes().to_vec());
                buffer.append(&mut buffered_act.to_le_bytes().to_vec());
                buffer.append(&mut act_id.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk89 { flag, arg } => {
                buffer.push(89);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::JumpInstall { flag, arg } => {
                buffer.push(92);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk94 { flag, arg, arg2 } => {
                buffer.push(94);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
                buffer.append(&mut arg2.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk95 { flag, arg1, arg2, arg3 } => {
                buffer.push(95);
                buffer.push(*flag);
                buffer.append(&mut arg1.to_le_bytes().to_vec());
                buffer.append(&mut arg2.to_le_bytes().to_vec());
                buffer.append(&mut arg3.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk96  { flag, arg1, arg2, arg3 } => {
                buffer.push(96);
                buffer.push(*flag);
                buffer.append(&mut arg1.to_le_bytes().to_vec());
                buffer.append(&mut arg2.to_le_bytes().to_vec());
                buffer.append(&mut arg3.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk99 { flag, arg } => {
                buffer.push(99);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk100 { flag, arg1, arg2 } => {
                buffer.push(100);
                buffer.push(*flag);
                buffer.append(&mut arg1.to_le_bytes().to_vec());
                buffer.append(&mut arg2.to_le_bytes().to_vec());
            }
            ScriptInstruction::SetAttackProperties {
                flag, arg, unk1, unk2, unk3, unk4 }
            => {
                buffer.push(101);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
                buffer.push(*unk1);
                buffer.push(*unk2);
                buffer.push(*unk3);
                buffer.push(*unk4);
            }
            ScriptInstruction::Unk132  { flag, arg } => {
                buffer.push(132);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::Unk133  { flag, arg } => {
                buffer.push(133);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::ScriptEnd  { flag, arg } => {
                buffer.push(253);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
            ScriptInstruction::ActionEnd  { flag, arg } => {
                buffer.push(255);
                buffer.push(*flag);
                buffer.append(&mut arg.to_le_bytes().to_vec());
            }
        }

        buffer
    }
}

#[binread]
#[derive(Clone, Serialize, Deserialize)]
pub struct ScriptAction {
    #[br(parse_with = until(|&inst| match inst {
        ScriptInstruction::ActionEnd { .. } => true,
        ScriptInstruction::ScriptEnd { .. } => true,
        _ => false,
    }))]
    pub instructions: Vec<ScriptInstruction>,
}
