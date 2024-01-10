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
        #[br(temp)]
        #[serde(skip)]
        flag: u8,
        #[br(temp)]
        #[serde(skip)]
        arg: u16,
    },
    #[br(magic(4u8))]
    RenewCollision {
        #[br(temp)]
        #[serde(skip)]
        flag: u8,
        #[br(temp)]
        #[serde(skip)]
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
    Move {
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
        #[br(temp)]
        #[serde(skip)]
        flag: u8,
        #[br(temp)]
        #[serde(skip)]
        arg: u16,
    },
    #[br(magic(17u8))]
    DrawReverse {
        #[br(temp)]
        #[serde(skip)]
        flag: u8,
        #[br(temp)]
        #[serde(skip)]
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
    Unk52 {
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
    Unk64 {
        flag: u8,
        arg1: u8,
        arg2: u8,
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
        #[br(temp)]
        #[serde(skip)]
        flag: u8,
        #[br(temp)]
        #[serde(skip)]
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
    Unk4B {
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
    #[br(magic(82u8))]
    Unk82 {
        flag: u8,
        magnitude: u16,
    },
    #[br(magic(84u8))]
    Pushback {
        flag: u8,
        magnitude: u16,
    },
    #[br(magic(85u8))]
    Stagger {
        flag: u8,
        magnitude: u16,
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
    Nop {
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
