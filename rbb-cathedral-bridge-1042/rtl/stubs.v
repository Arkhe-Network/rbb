module cellular_checkpoint (
    input  wire        clk,
    input  wire        rst_n,
    input  wire [15:0] theosis_level,
    input  wire        repair_done,
    output wire        clock_gated,
    output wire        g0_state,
    output wire [3:0]  checkpoint_phase
);
endmodule

module BUFGCE (
    input  wire I,
    input  wire CE,
    output wire O
);
endmodule
