import * as wasm from "../pkg/rust8";

let rom = "bgVlAGsGagCjDNqxegQ6QBIIewI7EhIGbCBtH6MQ3NEi9mAAYQCjEtARcAijDtARYEDwFfAHMAASNMYPZx5oAWn/ow7WcaMQ3NFgBOChfP5gBuChfAJgP4wC3NGjDtZxhoSHlGA/hgJhH4cSRx8SrEYAaAFGP2j/RwBpAdZxPwESqkcfEqpgBYB1PwASqmAB8BiAYGH8gBKjDNBxYP6JAyL2dQEi9kVgEt4SRmn/gGCAxT8BEsphAoAVPwES4IAVPwES7oAVPwES6GAg8BijDn7/gOCABGEA0BE+ABIwEt54/0j+aP8S7ngBSAJoAWAE8Bhp/xJwoxT1M/Jl8SljN2QA00VzBfIp00UA7uAAgAD8AKoAAAAAAA==";

var binary_data = window.atob(rom);
var data = new Uint8Array(new ArrayBuffer(binary_data.length));

for(var i = 0; i < binary_data.length; i++) {
	data[i] = binary_data.charCodeAt(i);
}

var chip8 = new wasm.Chip8();
chip8.load(new Uint8Array(data));

function run_loop() {
	chip8.tick();
	setTimeout(run_loop, 5);
}

run_loop();
