import * as wasm from "../pkg/rust8";

let rom =
  "bgVlAGsGagCjDNqxegQ6QBIIewI7EhIGbCBtH6MQ3NEi9mAAYQCjEtARcAijDtARYEDwFfAHMAASNMYPZx5oAWn/ow7WcaMQ3NFgBOChfP5gBuChfAJgP4wC3NGjDtZxhoSHlGA/hgJhH4cSRx8SrEYAaAFGP2j/RwBpAdZxPwESqkcfEqpgBYB1PwASqmAB8BiAYGH8gBKjDNBxYP6JAyL2dQEi9kVgEt4SRmn/gGCAxT8BEsphAoAVPwES4IAVPwES7oAVPwES6GAg8BijDn7/gOCABGEA0BE+ABIwEt54/0j+aP8S7ngBSAJoAWAE8Bhp/xJwoxT1M/Jl8SljN2QA00VzBfIp00UA7uAAgAD8AKoAAAAAAA==";

function load_rom() {
	let binary_data = window.atob(rom);
	var data = new Uint8Array(new ArrayBuffer(binary_data.length));

	for (var i = 0; i < binary_data.length; i++) {
	  data[i] = binary_data.charCodeAt(i);
	}
	return new Uint8Array(data);
}

var chip8 = new wasm.Chip8();
chip8.load(load_rom());

var canvas = document.getElementById("gameView");
var ctx = canvas.getContext("2d");

let pixel_width = Math.floor(canvas.width / 64);
let pixel_height = Math.floor(canvas.height / 32);

var window_padx = (canvas.width - pixel_width * 64) / 2;
var window_pady = (canvas.height - pixel_height * 32) / 2;

function build_grad(ctx, x, y, radius) {
  let grd = ctx.createRadialGradient(x, y, radius * 0.03, x, y, radius * 0.9);
  grd.addColorStop(0, "#00F200");
  grd.addColorStop(1, "black");
  return grd;
}

let use_window_gradient = true;

let wingrd = build_grad(ctx, canvas.width / 2, canvas.height / 2, canvas.width);
var previous_time = new Date().getTime();

function run_loop() {
  chip8.tick();

  ctx.fillStyle = "#000000";
  ctx.fillRect(window_padx, window_pady, pixel_width * 64, pixel_height * 32);

  for (var y = 0; y < 32; ++y) {
    for (var x = 0; x < 64; ++x) {
      let startX = window_padx + x * pixel_width;
      let startY = window_pady + y * pixel_height;

      if (chip8.check_pixel(x, y)) {
        ctx.fillStyle = use_window_gradient
          ? wingrd
          : build_grad(
              ctx,
              startX + pixel_width / 2,
              startY + pixel_height / 2,
              pixel_width
            );
        ctx.fillRect(startX, startY, pixel_width, pixel_height);
      }
    }
  }

  let current_time = new Date().getTime();
  chip8.update((current_time - previous_time) / 1000);
  previous_time = current_time;

  setTimeout(run_loop, 0);
}

run_loop();