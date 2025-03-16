module top (i_clk, i_switches, i_buttons, o_LEDs, i_RX, o_TX);
	input        i_clk      /*synthesis chip_pin = "AF14"*/;
	input  [9:0] i_switches /*synthesis chip_pin = "AE12", "AD10", "AC9", "AE11", "AD12, AD11, AF10, AF9, AC12, AB12"*/;
	input  [3:0] i_buttons  /*synthesis chip_pin = "Y16", "W15", "AA15", "AA14"*/;
	output [9:0] o_LEDs     /*synthesis chip_pin = "Y21, W21, W20, Y19, W19, W17, V18, V17, W16, V16"*/;
	input 		 i_RX       /*synthesis chip_pin = "AJ16*/; // Pin 10
	output 		 o_TX			/*synthesis chip_pin = "AJ17*/; // Pin 9
	
	`include "parameters.vh"
	
	wire [num_outputs-1:0] outputs;
	reg [num_outputs-1:0] outputs_reg;
	reg [num_inputs-1:0] inputs;
	
	// list of commands
	parameter CMD_send_outputs = 8'h01, CMD_change_input = 8'h02, CMD_reset = 8'hA5;
	
	// baud rate CLK
	wire baud;
	clk_div #(.limit(9), .n(12)) baud_clk (.ar(i_buttons[0]), .clk_in(i_clk), .clk_out(baud));
	
	// UART transmitter
	reg [7:0] tx_data;
	wire tx_done;
	reg send_data;
	wire ready;
	uart_transmit #(4'd8) transmitter (.i_clk(baud), .i_rst(i_buttons[0]), .i_start(send_data), .o_ready(ready), .o_done(tx_done), .tx_data(tx_data), .tx(o_TX)); 
	
	// UART reciver
	wire [7:0] rx_data;
	wire new_data;
	uart_receive #(4'd8) receiver (.i_clk(i_clk), .i_rst(i_buttons[0]), .o_done(new_data), .rx_data(rx_data), .rx(i_RX)); 
	
	// command controller
	parameter s_idle = 3'b000, s_send_bytes = 3'b001, s_wait_send = 3'b010, s_change_input = 3'b011, s_wait_receive = 3'b100, s_end = 3'b111;
	reg [2:0] state = s_idle;
	reg [7:0] byte_count;
	reg [input_id_len-1:0] input_id;
	
	always @(posedge baud or negedge i_buttons[0]) begin
		if (~i_buttons[0]) begin
			send_data = 1'b0;
			state = s_idle;
			byte_count = 8'b0;
		end
		else begin
			case (state)
				s_idle: begin
					if (new_data) begin
						case (rx_data)
							CMD_send_outputs: begin
								byte_count = num_o_bytes;
								outputs_reg = outputs;
								state = s_send_bytes;
							end
							CMD_change_input: begin
								byte_count = num_i_bytes;
								state = s_change_input;
							end
						endcase
					end
				end
				
				// CMD Send Bytes
				s_send_bytes: begin
					if (byte_count > 0) begin
						if (ready) begin
							tx_data = (outputs_reg >> ((num_o_bytes - byte_count) << 3)) & 8'hFF;
							send_data = 1'b1;
							byte_count = byte_count - 8'b1;
							state = s_wait_send;
						end
					end
					else begin
						state = s_end;
					end
				end
				s_wait_send: begin
					send_data = 1'b0;
					if (tx_done) begin
						state = s_send_bytes;
					end
				end
				
				// CMD Change Input
				s_change_input: begin
					if (byte_count > 0) begin
						if (~new_data) begin
							state = s_wait_receive;
						end
					end
					else begin
						inputs[input_id>>8] = input_id & 1'b1;
						state = s_end;
					end
				end
				s_wait_receive: begin
					if (new_data) begin
						byte_count = byte_count - 8'b1;
						input_id = (input_id << 8) | rx_data;
						state = s_change_input;
					end
				end
				
				s_end: begin
					if (~new_data) begin
						state = s_idle;
					end
				end
				default begin
					send_data = 1'b0;
					state = s_idle;
				end
			endcase 
		end
	end
	
	
	wire tick;
	
	wire [3:0] n1;
	wire [3:0] n2;
	wire [3:0] n3;
	wire [3:0] n4;
	
	assign n1 = 4'd4;
	assign n2 = 4'd8;
	assign n3 = 4'd7;
	assign n4 = 4'd10;
	
	wire [3:0] n12;
	wire [3:0] n34;
	
	wire [3:0] out;
	assign out = (n12 >= n34 ? n12 : n34);
	
	assign n12 = (n1 >= n2 ? n1 : n2);
	assign n34 = (n1 >= n2 ? n3 : n4);
	
	assign o_LEDs[3:0] = out;
	
	//clk_div #(.limit(2500000), .n(22)) tps (.ar(i_buttons[0]), .clk_in(i_clk), .clk_out(tick)); 
	
	Tick tick_clk ( .refclk(i_clk), .rst(~i_buttons[0]), .outclk_0(tick));
	
	redstone #(num_outputs, num_inputs) redstone (.tick(tick), .inputs(inputs), .outputs(outputs));
	
endmodule