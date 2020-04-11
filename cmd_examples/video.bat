..\target\release\chrono-photo ^
--pattern "../test_data/generated/image-*.jpg" ^
--frames ././1 ^
--video-in 0/5/2 ^
--video-out ././. ^
--temp-dir ../test_data/temp ^
--output ../test_data/out.jpg ^
--mode outlier ^
--threshold abs/0.05/0.2 ^
--background first ^
--outlier extreme ^
--quality 98 ^
--compression gzip/6 ^
--slice rows/2 ^
--debug
pause
