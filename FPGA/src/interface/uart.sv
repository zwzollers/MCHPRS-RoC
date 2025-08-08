module uart_rx #(
    parameter BAUD_DIVIDER_COUNT
)(
    input                i_clk,
    input                i_rx,
    output   wire        o_new_data,
    output   wire[7:0]   o_data
);
    localparam   
        s_IDLE      = 3'b000,
        s_START     = 3'b001,
        s_DATA      = 3'b010,
        s_STOP      = 3'b011,
        s_CLEANUP   = 3'b100;
    reg [2:0]   r_state     = s_IDLE;

    reg         r_rx_in     = 1'b1;
    reg         r_rx        = 1'b1;

    reg [7:0]   r_clk_cnt   = 0;
    reg [2:0]   r_bit       = 0;
    reg [7:0]   r_data      = 0;
    reg         r_new_data  = 0;
    

    // Purpose: Double-register the incoming data.
    // This allows it to be used in the UART RX Clock Domain.
    // (It removes problems caused by metastability)
    always @(posedge i_clk) begin
        r_rx_in <= i_rx;
        r_rx    <= r_rx_in;
    end


    // Purpose: Control RX state machine
    always @(posedge i_clk) begin
        case (r_state)
            s_IDLE : begin
                r_new_data          <= 1'b0;
                r_clk_cnt           <= 0;
                r_bit               <= 0;

                if (r_rx == 1'b0)
                    r_state         <= s_START;
                else
                    r_state         <= s_IDLE;
            end

            // Check middle of start bit to make sure it's still low
            s_START : begin
                if (r_clk_cnt == (BAUD_DIVIDER_COUNT-1)/2) begin
                    if (r_rx == 1'b0) begin
                        r_clk_cnt   <= 0;
                        r_state     <= s_DATA;
                    end
                    else
                        r_state     <= s_IDLE;
                end
                else begin
                    r_clk_cnt       <= r_clk_cnt + 1;
                    r_state         <= s_START;
                end
            end


            // Wait BAUD_DIVIDER_COUNT-1 clock cycles to sample serial data
            s_DATA : begin
                if (r_clk_cnt < BAUD_DIVIDER_COUNT-1) begin
                    r_clk_cnt       <= r_clk_cnt + 1;
                    r_state         <= s_DATA;
                end
                else begin
                    r_clk_cnt       <= 0;
                    r_data[r_bit]   <= r_rx;

                    // Check if we have received all bits
                    if (r_bit < 7) begin
                        r_bit       <= r_bit + 1;
                        r_state     <= s_DATA;
                    end
                    else begin
                        r_bit       <= 0;
                        r_state     <= s_STOP;                         
                    end
                end
            end
                
            s_STOP : begin
                // Wait BAUD_DIVIDER_COUNT-1 clock cycles for Stop bit to finish
                if (r_clk_cnt < BAUD_DIVIDER_COUNT-1) begin
                    r_clk_cnt       <= r_clk_cnt + 1;
                    r_state         <= s_STOP;                   
                end 
                else begin
                    r_new_data      <= 1'b1;
                    r_clk_cnt       <= 0;
                    r_state         <= s_CLEANUP;
                end
            end           

            s_CLEANUP : begin
                r_state             <= s_IDLE;
                r_new_data          <= 1'b0;
            end

            default :
                r_state             <= s_IDLE;

        endcase
    end   

    assign o_new_data   = r_new_data;
    assign o_data       = r_data;

endmodule

module uart_tx #(
    parameter BAUD_DIVIDER_COUNT
)(
    input               i_clk,
    input               i_start,
    input   wire[7:0]   i_data,
    output  wire        o_done, 
    output  wire        o_tx
);
    parameter   s_IDLE      = 2'b00;
    parameter   s_START     = 2'b01;
    parameter   s_DATA      = 2'b10;
    parameter   s_STOP      = 2'b11;
    reg [1:0]   r_state     = s_IDLE;

    reg         r_tx        = 1'b1;
    reg [3:0]   r_bit       = 4'd0;
    reg [7:0]   r_data      = 8'd0;
    reg [7:0]   r_baud_cnt  = 8'd0;

    reg         r_done      = 1'b0;

    reg         r_prev_start= 1'b0;
    wire        start;
    assign start = ~r_prev_start & i_start;

    always @(posedge i_clk) begin
        case (r_state)
            s_IDLE : begin
                r_done              <= 1'b0;
                r_bit               <= 4'd0;
                r_baud_cnt          <= 8'd0;

                if (start == 1'b1) begin
                    r_data          <= i_data;
                    r_state         <= s_START;
                end
                else
                    r_state         <= s_IDLE;
            end

            s_START : begin
                if (r_baud_cnt >= BAUD_DIVIDER_COUNT-1) begin
                    r_baud_cnt      <= 8'd0;
                    r_state         <= s_DATA;
                end
                else
                    r_tx            <= 1'b0;
                    r_baud_cnt      <= r_baud_cnt + 1;
            end

            s_DATA : begin
                if (r_baud_cnt >= BAUD_DIVIDER_COUNT-1) begin

                    if (r_bit == 4'd8) begin
                        r_state     <= s_STOP;
                    end
                    else begin
                        r_bit       <= r_bit + 1;
                        r_tx        <= r_data[r_bit];
                        r_baud_cnt  <= 8'd0;
                    end
                end
                else
                    r_baud_cnt      <= r_baud_cnt + 1;
            
            end
                
            s_STOP : begin
                if (r_baud_cnt >= BAUD_DIVIDER_COUNT*2) begin
                    r_baud_cnt      <= 8'd0;
                    r_done          <= 1'b1;
                    r_state         <= s_IDLE;
                end
                else
                    r_tx            <= 1'b1;
                    r_baud_cnt      <= r_baud_cnt + 1;
            end           

            default :
                r_state             <= s_IDLE;

        endcase
    end   

    assign o_tx     = r_tx;
    assign o_done   = r_done;

endmodule