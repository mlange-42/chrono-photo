..\target\release\chrono-photo ^
--pattern "../test_data/generated/image-*.jpg" ^
--frames ././1 ^
--temp-dir ../test_data/temp ^
--output ../test_data/out.jpg ^
--output-blend ../test_data/out-debug.png ^
--mode outlier ^
--threshold abs/0.05/0.2 ^
--background first ^
--outlier forward ^
--quality 98 ^
--fade repeat/abs/0,1/5,0 ^
--debug
pause
