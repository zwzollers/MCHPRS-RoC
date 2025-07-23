## Generated SDC file "RoC.out.sdc"

## Copyright (C) 2024  Intel Corporation. All rights reserved.
## Your use of Intel Corporation's design tools, logic functions 
## and other software and tools, and any partner logic 
## functions, and any output files from any of the foregoing 
## (including device programming or simulation files), and any 
## associated documentation or information are expressly subject 
## to the terms and conditions of the Intel Program License 
## Subscription Agreement, the Intel Quartus Prime License Agreement,
## the Intel FPGA IP License Agreement, or other applicable license
## agreement, including, without limitation, that your use is for
## the sole purpose of programming logic devices manufactured by
## Intel and sold by Intel or its authorized distributors.  Please
## refer to the applicable agreement for further details, at
## https://fpgasoftware.intel.com/eula.


## VENDOR  "Altera"
## PROGRAM "Quartus Prime"
## VERSION "Version 23.1std.1 Build 993 05/14/2024 SC Lite Edition"

## DATE    "Mon Jul 21 23:21:14 2025"

##
## DEVICE  "5CSEMA5F31C6"
##


#**************************************************************
# Time Information
#**************************************************************

set_time_format -unit ns -decimal_places 3



#**************************************************************
# Create Clock
#**************************************************************

create_clock -name {i_clk} -period 20.000 -waveform { 0.000 10.000 } [get_ports {i_clk}]


#**************************************************************
# Create Generated Clock
#**************************************************************

create_generated_clock -name {tick_clk} -source [get_nets {comb_4|tick_clk_inst|altera_pll_i|fboutclk_wire[0]}] -divide_by 2 -master_clock {i_clk} [get_registers {tps_clk_div:comb_5|o_clk}] 
create_generated_clock -name {comb_4|tick_clk_inst|altera_pll_i|general[0].gpll~FRACTIONAL_PLL|vcoph[0]} -source [get_pins {comb_4|tick_clk_inst|altera_pll_i|general[0].gpll~FRACTIONAL_PLL|refclkin}] -duty_cycle 50/1 -multiply_by 43 -divide_by 4 -master_clock {i_clk} [get_pins {comb_4|tick_clk_inst|altera_pll_i|general[0].gpll~FRACTIONAL_PLL|vcoph[0]}] 
create_generated_clock -name {comb_4|tick_clk_inst|altera_pll_i|general[0].gpll~PLL_OUTPUT_COUNTER|divclk} -source [get_pins {comb_4|tick_clk_inst|altera_pll_i|general[0].gpll~PLL_OUTPUT_COUNTER|vco0ph[0]}] -duty_cycle 50/1 -multiply_by 1 -master_clock {comb_4|tick_clk_inst|altera_pll_i|general[0].gpll~FRACTIONAL_PLL|vcoph[0]} [get_pins {comb_4|tick_clk_inst|altera_pll_i|general[0].gpll~PLL_OUTPUT_COUNTER|divclk}] 
create_generated_clock -name {uart_clk} -source [get_ports {i_clk}] -divide_by 20 -master_clock {i_clk} [get_registers {command_controller:cmd_ctrl|uart_tx:tx|clk_div:baud_clk_div|r_clk}] 


#**************************************************************
# Set Clock Latency
#**************************************************************



#**************************************************************
# Set Clock Uncertainty
#**************************************************************

set_clock_uncertainty -rise_from [get_clocks {i_clk}] -rise_to [get_clocks {i_clk}] -setup 0.310  
set_clock_uncertainty -rise_from [get_clocks {i_clk}] -rise_to [get_clocks {i_clk}] -hold 0.270  
set_clock_uncertainty -rise_from [get_clocks {i_clk}] -fall_to [get_clocks {i_clk}] -setup 0.310  
set_clock_uncertainty -rise_from [get_clocks {i_clk}] -fall_to [get_clocks {i_clk}] -hold 0.270  
set_clock_uncertainty -rise_from [get_clocks {i_clk}] -rise_to [get_clocks {comb_4|tick_clk_inst|altera_pll_i|general[0].gpll~PLL_OUTPUT_COUNTER|divclk}]  0.270  
set_clock_uncertainty -rise_from [get_clocks {i_clk}] -fall_to [get_clocks {comb_4|tick_clk_inst|altera_pll_i|general[0].gpll~PLL_OUTPUT_COUNTER|divclk}]  0.270  
set_clock_uncertainty -rise_from [get_clocks {i_clk}] -rise_to [get_clocks {tick_clk}]  0.330  
set_clock_uncertainty -rise_from [get_clocks {i_clk}] -fall_to [get_clocks {tick_clk}]  0.330  
set_clock_uncertainty -rise_from [get_clocks {i_clk}] -rise_to [get_clocks {Uart}] -setup 0.360  
set_clock_uncertainty -rise_from [get_clocks {i_clk}] -rise_to [get_clocks {Uart}] -hold 0.320  
set_clock_uncertainty -rise_from [get_clocks {i_clk}] -fall_to [get_clocks {Uart}] -setup 0.360  
set_clock_uncertainty -rise_from [get_clocks {i_clk}] -fall_to [get_clocks {Uart}] -hold 0.320  
set_clock_uncertainty -fall_from [get_clocks {i_clk}] -rise_to [get_clocks {i_clk}] -setup 0.310  
set_clock_uncertainty -fall_from [get_clocks {i_clk}] -rise_to [get_clocks {i_clk}] -hold 0.270  
set_clock_uncertainty -fall_from [get_clocks {i_clk}] -fall_to [get_clocks {i_clk}] -setup 0.310  
set_clock_uncertainty -fall_from [get_clocks {i_clk}] -fall_to [get_clocks {i_clk}] -hold 0.270  
set_clock_uncertainty -fall_from [get_clocks {i_clk}] -rise_to [get_clocks {comb_4|tick_clk_inst|altera_pll_i|general[0].gpll~PLL_OUTPUT_COUNTER|divclk}]  0.270  
set_clock_uncertainty -fall_from [get_clocks {i_clk}] -fall_to [get_clocks {comb_4|tick_clk_inst|altera_pll_i|general[0].gpll~PLL_OUTPUT_COUNTER|divclk}]  0.270  
set_clock_uncertainty -fall_from [get_clocks {i_clk}] -rise_to [get_clocks {tick_clk}]  0.330  
set_clock_uncertainty -fall_from [get_clocks {i_clk}] -fall_to [get_clocks {tick_clk}]  0.330  
set_clock_uncertainty -fall_from [get_clocks {i_clk}] -rise_to [get_clocks {Uart}] -setup 0.360  
set_clock_uncertainty -fall_from [get_clocks {i_clk}] -rise_to [get_clocks {Uart}] -hold 0.320  
set_clock_uncertainty -fall_from [get_clocks {i_clk}] -fall_to [get_clocks {Uart}] -setup 0.360  
set_clock_uncertainty -fall_from [get_clocks {i_clk}] -fall_to [get_clocks {Uart}] -hold 0.320  
set_clock_uncertainty -rise_from [get_clocks {comb_4|tick_clk_inst|altera_pll_i|general[0].gpll~PLL_OUTPUT_COUNTER|divclk}] -rise_to [get_clocks {comb_4|tick_clk_inst|altera_pll_i|general[0].gpll~PLL_OUTPUT_COUNTER|divclk}] -setup 0.100  
set_clock_uncertainty -rise_from [get_clocks {comb_4|tick_clk_inst|altera_pll_i|general[0].gpll~PLL_OUTPUT_COUNTER|divclk}] -rise_to [get_clocks {comb_4|tick_clk_inst|altera_pll_i|general[0].gpll~PLL_OUTPUT_COUNTER|divclk}] -hold 0.060  
set_clock_uncertainty -rise_from [get_clocks {comb_4|tick_clk_inst|altera_pll_i|general[0].gpll~PLL_OUTPUT_COUNTER|divclk}] -fall_to [get_clocks {comb_4|tick_clk_inst|altera_pll_i|general[0].gpll~PLL_OUTPUT_COUNTER|divclk}] -setup 0.100  
set_clock_uncertainty -rise_from [get_clocks {comb_4|tick_clk_inst|altera_pll_i|general[0].gpll~PLL_OUTPUT_COUNTER|divclk}] -fall_to [get_clocks {comb_4|tick_clk_inst|altera_pll_i|general[0].gpll~PLL_OUTPUT_COUNTER|divclk}] -hold 0.060  
set_clock_uncertainty -rise_from [get_clocks {comb_4|tick_clk_inst|altera_pll_i|general[0].gpll~PLL_OUTPUT_COUNTER|divclk}] -rise_to [get_clocks {tick_clk}]  0.290  
set_clock_uncertainty -rise_from [get_clocks {comb_4|tick_clk_inst|altera_pll_i|general[0].gpll~PLL_OUTPUT_COUNTER|divclk}] -fall_to [get_clocks {tick_clk}]  0.290  
set_clock_uncertainty -fall_from [get_clocks {comb_4|tick_clk_inst|altera_pll_i|general[0].gpll~PLL_OUTPUT_COUNTER|divclk}] -rise_to [get_clocks {comb_4|tick_clk_inst|altera_pll_i|general[0].gpll~PLL_OUTPUT_COUNTER|divclk}] -setup 0.100  
set_clock_uncertainty -fall_from [get_clocks {comb_4|tick_clk_inst|altera_pll_i|general[0].gpll~PLL_OUTPUT_COUNTER|divclk}] -rise_to [get_clocks {comb_4|tick_clk_inst|altera_pll_i|general[0].gpll~PLL_OUTPUT_COUNTER|divclk}] -hold 0.060  
set_clock_uncertainty -fall_from [get_clocks {comb_4|tick_clk_inst|altera_pll_i|general[0].gpll~PLL_OUTPUT_COUNTER|divclk}] -fall_to [get_clocks {comb_4|tick_clk_inst|altera_pll_i|general[0].gpll~PLL_OUTPUT_COUNTER|divclk}] -setup 0.100  
set_clock_uncertainty -fall_from [get_clocks {comb_4|tick_clk_inst|altera_pll_i|general[0].gpll~PLL_OUTPUT_COUNTER|divclk}] -fall_to [get_clocks {comb_4|tick_clk_inst|altera_pll_i|general[0].gpll~PLL_OUTPUT_COUNTER|divclk}] -hold 0.060  
set_clock_uncertainty -fall_from [get_clocks {comb_4|tick_clk_inst|altera_pll_i|general[0].gpll~PLL_OUTPUT_COUNTER|divclk}] -rise_to [get_clocks {tick_clk}]  0.290  
set_clock_uncertainty -fall_from [get_clocks {comb_4|tick_clk_inst|altera_pll_i|general[0].gpll~PLL_OUTPUT_COUNTER|divclk}] -fall_to [get_clocks {tick_clk}]  0.290  
set_clock_uncertainty -rise_from [get_clocks {tick_clk}] -rise_to [get_clocks {i_clk}]  0.330  
set_clock_uncertainty -rise_from [get_clocks {tick_clk}] -fall_to [get_clocks {i_clk}]  0.330  
set_clock_uncertainty -rise_from [get_clocks {tick_clk}] -rise_to [get_clocks {tick_clk}] -setup 0.280  
set_clock_uncertainty -rise_from [get_clocks {tick_clk}] -rise_to [get_clocks {tick_clk}] -hold 0.270  
set_clock_uncertainty -rise_from [get_clocks {tick_clk}] -fall_to [get_clocks {tick_clk}] -setup 0.280  
set_clock_uncertainty -rise_from [get_clocks {tick_clk}] -fall_to [get_clocks {tick_clk}] -hold 0.270  
set_clock_uncertainty -fall_from [get_clocks {tick_clk}] -rise_to [get_clocks {i_clk}]  0.330  
set_clock_uncertainty -fall_from [get_clocks {tick_clk}] -fall_to [get_clocks {i_clk}]  0.330  
set_clock_uncertainty -fall_from [get_clocks {tick_clk}] -rise_to [get_clocks {tick_clk}] -setup 0.280  
set_clock_uncertainty -fall_from [get_clocks {tick_clk}] -rise_to [get_clocks {tick_clk}] -hold 0.270  
set_clock_uncertainty -fall_from [get_clocks {tick_clk}] -fall_to [get_clocks {tick_clk}] -setup 0.280  
set_clock_uncertainty -fall_from [get_clocks {tick_clk}] -fall_to [get_clocks {tick_clk}] -hold 0.270  
set_clock_uncertainty -rise_from [get_clocks {Uart}] -rise_to [get_clocks {i_clk}] -setup 0.360  
set_clock_uncertainty -rise_from [get_clocks {Uart}] -rise_to [get_clocks {i_clk}] -hold 0.320  
set_clock_uncertainty -rise_from [get_clocks {Uart}] -fall_to [get_clocks {i_clk}] -setup 0.360  
set_clock_uncertainty -rise_from [get_clocks {Uart}] -fall_to [get_clocks {i_clk}] -hold 0.320  
set_clock_uncertainty -rise_from [get_clocks {Uart}] -rise_to [get_clocks {Uart}] -setup 0.410  
set_clock_uncertainty -rise_from [get_clocks {Uart}] -rise_to [get_clocks {Uart}] -hold 0.380  
set_clock_uncertainty -rise_from [get_clocks {Uart}] -fall_to [get_clocks {Uart}] -setup 0.410  
set_clock_uncertainty -rise_from [get_clocks {Uart}] -fall_to [get_clocks {Uart}] -hold 0.380  
set_clock_uncertainty -fall_from [get_clocks {Uart}] -rise_to [get_clocks {i_clk}] -setup 0.360  
set_clock_uncertainty -fall_from [get_clocks {Uart}] -rise_to [get_clocks {i_clk}] -hold 0.320  
set_clock_uncertainty -fall_from [get_clocks {Uart}] -fall_to [get_clocks {i_clk}] -setup 0.360  
set_clock_uncertainty -fall_from [get_clocks {Uart}] -fall_to [get_clocks {i_clk}] -hold 0.320  
set_clock_uncertainty -fall_from [get_clocks {Uart}] -rise_to [get_clocks {Uart}] -setup 0.410  
set_clock_uncertainty -fall_from [get_clocks {Uart}] -rise_to [get_clocks {Uart}] -hold 0.380  
set_clock_uncertainty -fall_from [get_clocks {Uart}] -fall_to [get_clocks {Uart}] -setup 0.410  
set_clock_uncertainty -fall_from [get_clocks {Uart}] -fall_to [get_clocks {Uart}] -hold 0.380  


#**************************************************************
# Set Input Delay
#**************************************************************



#**************************************************************
# Set Output Delay
#**************************************************************



#**************************************************************
# Set Clock Groups
#**************************************************************



#**************************************************************
# Set False Path
#**************************************************************



#**************************************************************
# Set Multicycle Path
#**************************************************************



#**************************************************************
# Set Maximum Delay
#**************************************************************



#**************************************************************
# Set Minimum Delay
#**************************************************************



#**************************************************************
# Set Input Transition
#**************************************************************

