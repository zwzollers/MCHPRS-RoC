module uart_transmit (i_clk, i_rst, i_start, o_ready, o_done, tx_data, tx);

    input                 i_clk;
    input                 i_rst;
    input                 i_start;
	 output reg            o_ready;
	 output reg            o_done;
    input [data_bits-1:0] tx_data;
    output reg            tx;

    parameter data_bits = 4'd8;

    parameter s_idle = 2'b00, s_start = 2'b01, s_data = 2'b10, s_end = 2'b11; 
    reg [1:0] state = s_idle;
    reg [3:0] bit_count = 4'd0;
	 
	 

    initial begin
        tx <= 1'b1;
    end

    always @(posedge i_clk or negedge i_rst) begin
		if (~i_rst) begin
            state = s_idle;
            bit_count = 4'd0;
            tx = 1'b1;
				o_ready = 1'b0;
				o_done = 1'b0;
        end
        else begin
            case (state)
                s_idle: begin
                    if (i_start == 1'b1) begin
                        state = s_start;
                        tx = 1'b0;
								o_ready = 1'b0;
								o_done = 1'b0;
                    end
						  else begin
							o_ready = 1'b1;
						end
                end
                s_start: begin
						  state = s_data;
                    tx = tx_data[0];
                    bit_count = 4'd1;
                end
                s_data: begin
						   if (bit_count >= data_bits) begin
							 	 state = s_end;
								 tx = 1'b1;
								 o_done = 1'b1;
							end
							else begin
								 tx = tx_data[bit_count];
								 bit_count = bit_count + 4'b1;
							end
                end
                s_end: begin
                    if (~i_start) begin
								o_ready = 1'b1;
                        state = s_idle;
                    end
                end
                default: begin
                     state = s_idle;
							bit_count = 4'd0;
							tx = 1'b1;
							o_ready = 1'b0;
							o_done = 1'b0;	
                end
            endcase
        end
	end
endmodule

module uart_receive (i_clk, i_rst, o_done, rx_data, rx);
	
	input                      i_clk;
	input                      i_rst;
	output reg                 o_done;
	output reg [data_bits-1:0] rx_data;
	input                      rx;

	parameter data_bits = 4'd8;

	parameter s_idle = 2'b00, s_start = 2'b01, s_data = 2'b10, s_end = 2'b11; 
	reg [1:0] state = s_idle;
	reg [3:0] bit_count = 4'd0;
	
	reg [4:0] clock_count = 4'b0;
	reg [2:0] syncr = 3'b0;
	reg rx_sync;
	
	always @(posedge i_clk) begin
		syncr = {syncr[1:0],rx};
		rx_sync = syncr[2];
	end

	always @(posedge i_clk or negedge i_rst) begin
		if (~i_rst) begin
		state = s_idle;
		bit_count = 4'd0;
		o_done = 1'b0;
		clock_count = 4'b0;
		end
		else begin
			case (state)
				s_idle: begin
					if (~rx_sync) begin
						state = s_start;
						bit_count = 4'd0;
						o_done = 1'b0;
					end
				end
				s_start: begin
					if (clock_count >= 5'd29) begin
						state = s_data;
					end
					else begin
						clock_count = clock_count + 1'b1;
					end
				end
				s_data: begin
					if (clock_count >= 5'd19) begin
						clock_count = 4'b0;
						if (bit_count >= data_bits) begin
							o_done = 1'b1;
							state = s_end;
						end
						else begin
							rx_data[bit_count] = rx_sync;
							bit_count = bit_count + 4'b1;
						end
					end
					else begin
						clock_count = clock_count + 1'b1;
					end
				end
				s_end: begin
					state = s_idle;
				end
				default: begin
					state = s_idle;
					bit_count = 4'd0;
					o_done = 1'b0;
					clock_count = 4'b0;
				end
			endcase
		end
	end
	
endmodule 
