/**
 * Disassembles basic 8086 binaries into valid 8086 ASM files
 */

use std::{env, fs};

const REG_NAMES: [[&str; 8]; 2] = [
    ["al","cl","dl","bl","ah","ch","dh","bh"], //w = 0
    ["ax","cx","dx","bx","sp","bp","si","di"]  //w = 1
];

//R/M = 110 is Direct Address and has 16 bit displacement
const EA_CALC: [&str; 8] = [
    "bx + si", "bx + di", "bp + si", "bp + di", 
    "si", "di","bp", "bx"];
    
const MOV_MASKS: [u8;7] = [0b10001000, 0b11000110, 0b10110000, 0b10100000, 0b10100010, 0b10001110, 0b10001100];

fn main() {
    //READ IN FILE
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];
    let contents = fs::read(file_path).expect("Unable to read file");

    let mut lines = vec![String::from("bits 16")];
    //BEGIN PARSING
    let mut count = 0;

    while count < contents.len() {
        //first byte/loop will always be an OP Code
        //op code will determine if 1 or more additional bytes need to be read
        //2nd byte if used will determine if a 1-4 more bytes need to be read
        let op_code = OpCode::build(contents[count]);
        let (displacment, asm) = to_asm(op_code, count, &contents);

        lines.push(asm);
        count += displacment + 1;
    }

    //print assembly to stdout at the end
    for line in lines {
        println!("{line}");
    }
}

/**
 * code
 * d: 0 = destination REG, 1 = source REG
 * w: 0 = byte, 1 = word
 */
#[derive(Debug)]
#[warn(dead_code)]
struct OpCode {
    displacement: usize,
    op_type: OpType,
    d: u8, 
    w: u8,
    // s: u8,
    // v: u8,
    // z: u8,
    reg: u8
}

impl OpCode {
    fn build(code: u8) -> OpCode {
        let mut displacement = 0;
        let mut op_type = OpType::NOP;
        let mut d= 0;
        let mut w= 0;
        // let s= 0;
        // let v= 0;
        // let z= 0;
        let mut reg = 0;
        //MOV codes
        let mut index = 0;
        while index < MOV_MASKS.len() {
            let op = match index {
                0 => code >> 2 << 2,
                1|3|4 => code >> 1 << 1,
                2 => code >> 4 << 4,
                5|6 => code,
                _ => unreachable!("Index too high")
            };
            if op == MOV_MASKS[index] {
                // better way to pattern match on this?
                match index {
                    0 => {
                        
                        op_type = OpType::MovRegmemTofromReg;
                        d = (0b00000010u8 & code) >> 1;
                        w = 0b1u8 & code;
                        displacement = 1;
                        break;
                    },
                    1 => {
                        op_type = OpType::MovImmToRegmem;
                        displacement = 1;
                        w = 0b1u8 & code;
                        break;
                    },
                    2 => {
                        op_type = OpType::MovImmToReg;
                        w = (0b00001000u8 & code) >> 3;
                        if w == 1 {
                            displacement = 2;
                        } else {
                            displacement = 1;
                        }
                        reg = 0b00000111u8 & code;
                        break;
                    },
                    3 => {
                        op_type = OpType::MovMemToAcc;
                        displacement = 1;
                        w = 0b1u8 & code;
                        break;
                    },
                    4 => {
                        op_type = OpType::MovAccToMem;
                        displacement = 1;
                        w = 0b1u8 & code;
                        break;
                    },
                    5 => unimplemented!("Not Implemented"),
                    6 => unimplemented!("Not Implemented"),
                    _ => unimplemented!("Found Unimplemented code")
                }
            }
            index += 1;
        }
        return OpCode {displacement, op_type, d,w, reg };
    }
}

#[derive(Debug)]
enum OpType {
    MovRegmemTofromReg,
    MovImmToRegmem,
    MovImmToReg,
    MovMemToAcc,
    MovAccToMem,
    //MovRegmemToSr,
    //MovSrToRegmem,
    NOP
}

impl OpType {
    fn to_string(&self) -> &str {
        match self {
            OpType::MovRegmemTofromReg|
            OpType::MovImmToRegmem|
            OpType::MovImmToReg|
            OpType::MovMemToAcc|
            OpType::MovAccToMem
            //OpType::MovRegmemToSr| OpType::MovSrToRegmem 
            => "mov",
            _ => unimplemented!("Found Unimplemented OP Type")
        }
    }
}

//Returns a string of valid ASM Syntax
//Return number of bytes processed
fn to_asm(op_code: OpCode, count: usize, contents: &Vec<u8>) -> (usize, String) {
    let mut bytes_to_process = op_code.displacement;
    let mut asm = String::new();
    if op_code.displacement > 0 {
        match op_code.op_type  {
            OpType::MovRegmemTofromReg => {
                let byte2 = contents[count + 1];
                let mode = (0b11000000u8 & byte2) >> 6;
                let reg = (0b00111000u8 & byte2) >> 3;
                let rm = 0b00000111u8 & byte2;
                match mode {
                    0 => {
                        if rm == 7 {
                            bytes_to_process += 2;
                        } else {
                            match op_code.d {
                                0 => {
                                    let dest = REG_NAMES[op_code.w as usize][reg as usize];
                                    let src = EA_CALC[rm as usize];
                                    asm = format!("{} [{}], {}", op_code.op_type.to_string(),src,dest);
                                },
                                1 => {
                                    let src = REG_NAMES[op_code.w as usize][rm as usize];
                                    let dest = EA_CALC[rm as usize];
                                    asm = format!("{} {}, [{}]", op_code.op_type.to_string(),src,dest);
                                },
                                _ => unreachable!("Invalid Instruction found")
                            }
                        }
                    },
                    1 => {
                        bytes_to_process += 1;
                        match op_code.d {
                            0 => {
                                let dest = REG_NAMES[op_code.w as usize][reg as usize];
                                let src = EA_CALC[rm as usize];
                                //let data: u8 = contents[count +2];
                                asm = format!("{} [{}], {}", op_code.op_type.to_string(),src,dest);
                            },
                            1 => {
                                let src = REG_NAMES[op_code.w as usize][reg as usize];
                                let dest = EA_CALC[rm as usize];
                                let data: u8 = contents[count +2];
                                if data > 0 {
                                    asm = format!("{} {}, [{} + {}]", op_code.op_type.to_string(),src,dest, data);    
                                } else{
                                    asm = format!("{} {}, [{}]", op_code.op_type.to_string(),src,dest);
                                }
                                
                            },
                            _ => unreachable!("Invalid Instruction found")
                        }
                    },
                    2 => {
                        bytes_to_process += 2;
                        match op_code.d {
                            0 => {
                                let dest = REG_NAMES[op_code.w as usize][rm as usize];
                                let src = EA_CALC[reg as usize];
                                
                                asm = format!("{} {}, [{}]", op_code.op_type.to_string(),dest,src);
                            },
                            1 => {
                                let src = REG_NAMES[op_code.w as usize][rm as usize];
                                let dest = EA_CALC[reg as usize];
                                let mut data: u16 = (contents[count + 3] as u16) << 8;
                                data = data | contents[count +2] as u16;
                                asm = format!("{} {}, [{} + {}]", op_code.op_type.to_string(),src,dest, data);
                            },
                            _ => unreachable!("Invalid Instruction found")
                        }
                    },
                    3 => {
                        match op_code.d {
                            // destination
                            0 => {
                                let dest = REG_NAMES[op_code.w as usize][rm as usize];
                                let src = REG_NAMES[op_code.w as usize][reg as usize];
                                asm = format!("{} {}, {}", op_code.op_type.to_string(), dest, src);
                            },
                            //source
                            1 => println!("Not Implemented: {:?}, mode: {mode}, reg: {reg}, rm: {rm}",op_code),
                            _ => unreachable!("Invalid Instruction found")
                        }
                    },
                    _ => unreachable!("Invalid Instruction")
                }
            },
            OpType::MovImmToReg => {
                let dest = REG_NAMES[op_code.w as usize][op_code.reg as usize];
                let mut data = contents[count + 1] as u16;
                if op_code.w == 1 {
                    let hi: u16 = (contents[count + 2] as u16) << 8;
                    data = hi | data;
                }
                asm = format!("{} {}, {}", op_code.op_type.to_string(), dest, data);
            },
            _ => println!("Not implemented: {:?}", op_code)
        }
    }
    (bytes_to_process, asm)
}
