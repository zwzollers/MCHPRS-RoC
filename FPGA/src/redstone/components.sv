module repeater #(
	parameter 	t,
	parameter 	state,
	parameter 	lock_out,
	parameter 	lockable
) (
	input 		i_clk,
	input 		i_in,
	input 		i_lock,
	output 		o_out
);

	reg [t-1:0] buffer = {t{state}};
	
	generate
	
		if (lock_out == 0 && lockable == 0 && t == 1) begin
			assign o_out = buffer[t-1];
			always @(posedge i_clk) begin
				buffer <= i_in;
			end
		end
		
		else if (lock_out == 0 && lockable == 0 && t > 1) begin
			assign o_out = buffer[t-1];
			always @(posedge i_clk) begin
				buffer <= {buffer[t-2:0] | {t-1{buffer[t-1] & i_in}}, i_in | (~buffer[t-1] & buffer[0])};
			end
		end
		
		else if (lock_out == 1 && t == 1) begin
			assign o_out = buffer[t-1] | i_in;
			always @(posedge i_clk) begin
				buffer <= i_in;
			end
		end
		
		//fix
		else if (lock_out == 1 && t > 1) begin
			assign o_out = buffer[t-2] | (buffer[t-1] & i_in);
			always @(posedge i_clk) begin
				buffer <= {buffer[t-2:0] | {t-1{buffer[t-1] & i_in}}, i_in | (~buffer[t-1] & buffer[0])};
			end
		end
				
		else if (lockable == 1 && t == 1) begin
			assign o_out = buffer[t-1];
			always @(posedge i_clk) begin
				if (~i_lock)
					buffer <= i_in;
				else 
					buffer <= buffer;
			end
		end
		
		else if (lockable == 1 && t > 1) begin 
			assign o_out = buffer[t-1];
			always @(posedge i_clk) begin
				if (i_lock)
					buffer <= {t{buffer[t-1]}};
				else
					buffer <= {buffer[t-2:0] | {t-1{buffer[t-1] & i_in}}, i_in | (~buffer[t-1] & buffer[0])};
			end
		end
	endgenerate

endmodule

module torch #(
	parameter 	state
) (
	input 		i_clk,
	input 		i_in,
	output 		o_out
);

	reg buffer = state;

	assign o_out = ~buffer;

	always @(posedge i_clk) begin
		buffer <= i_in;
	end

endmodule

module comp #(
	parameter 	size,
	parameter 	state

) (
	input 				i_clk,
	input 	[size-1:0]	i_in,
	output 	[size-1:0]	o_out
);

	reg [size-1:0] buffer = state;

	assign o_out = buffer;

	always @(posedge i_clk) begin
		buffer <= i_in;
	end
			
endmodule
