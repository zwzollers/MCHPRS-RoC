
module    clk_div(ar, clk_in, clk_out);
    input ar;
    input clk_in;         // 50 MHz for audio codec on DE2 board
    output reg clk_out;       // 
    
    //reg   clk_out;
    parameter n = 15;      // Bit width of counter and limit (log2 of limit)
    parameter [n-1:0] limit = 15'd24999; // 32 counts from 992-1023
    reg [n-1:0]   count;
    
    
    always @(negedge ar or posedge clk_in)
    if(~ar)
       begin
          count = 0;
		  clk_out = 1'b0;
       end
    else
      begin

       if(count >= limit)  // use inequalty to save logic space for comparison
         begin
		 clk_out = ~clk_out;
         
		 count = 0;
		 end
		else
			count = count + 1;
      end

 endmodule  
                 
    
   