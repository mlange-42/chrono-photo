..\target\release\chrono-photo ^
--pattern ../test_data/test_16bit/*.tif ^
--frames ././1 ^
--temp-dir ../test_data/temp ^
--output ../test_data/out.tif ^
--output-blend ../test_data/out-debug.png ^
--mode outlier ^
--threshold abs/0.05/0.2 ^
--background random ^
--outlier extreme ^
--quality 98 ^
--debug
pause
