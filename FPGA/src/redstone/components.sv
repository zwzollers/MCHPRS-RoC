module repeater (i_clk, i_in, i_lock, o_out);

	input  i_clk;
	input  i_in;
	input  i_lock;
	output o_out;

	parameter 
		t = 1,
	    state = 1'b0,
	    lock_out = 0,
	    lockable = 0;

	reg [t-1:0] buffer = {t{state}};
	
	generate
	
		if (lock_out == 0 && lockable == 0 && t == 1) begin
			assign o_out = buffer[t-1];
			always @(posedge i_clk) begin
				buffer = i_in;
			end
		end
		
		else if (lock_out == 0 && lockable == 0 && t > 1) begin
			assign o_out = buffer[t-1];
			always @(posedge i_clk) begin
				buffer = {buffer[t-2:0] | {t-1{buffer[t-1] & i_in}}, i_in | (~buffer[t-1] & buffer[0])};
			end
		end
		
		else if (lock_out == 1 && t == 1) begin
			assign o_out = buffer[t-1] | i_in;
			always @(posedge i_clk) begin
				buffer = i_in;
			end
		end
		
		//fix
		else if (lock_out == 1 && t > 1) begin
			assign o_out = buffer[t-2] | (buffer[t-1] & i_in);
			always @(posedge i_clk) begin
				buffer = {buffer[t-2:0] | {t-1{buffer[t-1] & i_in}}, i_in | (~buffer[t-1] & buffer[0])};
			end
		end
				
		else if (lockable == 1 && t == 1) begin
			assign o_out = buffer[t-1];
			always @(posedge i_clk) begin
				buffer = (i_lock & buffer) | (!i_lock & i_in);
			end
		end
		
		else if (lockable == 1 && t > 1) begin 
			assign o_out = buffer[t-1];
			always @(posedge i_clk) begin
				if (i_lock)
					buffer = {t{buffer[t-1]}};
				else
					buffer = {buffer[t-2:0] | {t-1{buffer[t-1] & i_in}}, i_in | (~buffer[t-1] & buffer[0])};
			end
		end
	endgenerate
endmodule

module torch (i_clk, i_in, o_out);

	input  i_clk;
	input  i_in;
	output o_out;
	
	parameter state = 1'b0;

	reg buffer = state;

	assign o_out = ~buffer;

	always @(posedge i_clk) begin
		buffer = i_in;
	end

endmodule

module comp (i_clk, i_in, i_side, o_out);

	input 		 i_clk;
	input  [3:0] i_in;
	input  [3:0] i_side;

	output [3:0] o_out;

	reg [3:0] buffer = state;
	assign o_out = buffer;

	parameter 
		state = 4'd0,
		mode = 0;

	generate 
		if (mode == 0) begin
			always @(posedge i_clk) begin
				buffer = (i_in >= i_side) ? i_in : 4'd0;
			end
		end 
		else begin
			always @(posedge i_clk) begin
				buffer = (i_in >= i_side) ? (i_in - i_side) : 4'd0;
			end
		end
	endgenerate
			

endmodule
