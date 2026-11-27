# WLF fixture preparation

Set `QUESTA_HOME` as described in the packaged Quickstart. The optional reader uses the SDK header and library. Test and benchmark preparation also requires `vcd2wlf`.

`just prepare-wlf-test-fixtures` converts small VCD regression inputs. `just prepare-wlf-fixtures` prepares inputs for the benchmark catalog. FST conversion streams the waveform through VCD. The converter maps extended declaration kinds to standard VCD types with the same value encoding and width. Native fixtures cover data that the VCD importer cannot represent.
