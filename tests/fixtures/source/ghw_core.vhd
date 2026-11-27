library ieee;
use ieee.std_logic_1164.all;

entity ghw_core is
end entity;

architecture fixture of ghw_core is
begin
  dut : block
    type detail_t is record
      marker : std_logic;
    end record;

    signal valid : std_logic := '0';
    signal data : std_logic_vector(7 downto 0) := x"00";
    signal detail : detail_t := (marker => '0');
  begin
    stimulus : process
    begin
      wait for 5 ns;
      valid <= '1';
      data <= x"3C";
      wait for 5 ns;
      valid <= '0';
      data <= x"A5";
      detail.marker <= '1';
      wait;
    end process;
  end block;
end architecture;
