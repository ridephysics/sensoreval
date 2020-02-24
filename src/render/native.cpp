#include <stdio.h>
#include <stdint.h>
#include <chrono>
#include <iostream>
#include <opencv2/core.hpp>
#include <opencv2/videoio.hpp>
#include <opencv2/highgui/highgui.hpp>

typedef int (*render_hud_t)(void *context, uint8_t *buf, size_t w, size_t h);

extern "C" int sensoreval_render_native_render_video(
    const char *dst,
    const char *src,
    render_hud_t render_hud,
    void *ctx)
{
    int rc;
    static const char *window = "frame";
    //cv::namedWindow(window, cv::WINDOW_AUTOSIZE);

    cv::VideoCapture inputVideo(src);
    if (!inputVideo.isOpened()) {
        fprintf(stderr, "couldn't open the input video: %s\n", src);
        return -1;
    }

    double width = inputVideo.get(cv::VideoCaptureProperties::CAP_PROP_FRAME_WIDTH);
    if (width == 0) {
        fprintf(stderr, "can't get frame width\n");
        return -1;
    }

    double height = inputVideo.get(cv::VideoCaptureProperties::CAP_PROP_FRAME_HEIGHT);
    if (height == 0) {
        fprintf(stderr, "can't get frame height\n");
        return -1;
    }

    double fps = inputVideo.get(cv::VideoCaptureProperties::CAP_PROP_FPS);
    if (fps == 0) {
        fprintf(stderr, "can't get fps\n");
        return -1;
    }

    cv::Mat frame;
    cv::Mat hud(height, width, CV_8UC4);

    cv::VideoWriter writer;
    int codec = cv::VideoWriter::fourcc('X', '2', '6', '4');
    writer.open(dst, codec, fps, cv::Size(width, height), true);
    if (!writer.isOpened()) {
        std::cerr << "Could not open the output video file for write\n";
        return -1;
    }

    auto start = std::chrono::steady_clock::now();
    for (size_t i=0; i<1000; i++) {
        inputVideo >> frame;
        writer.write(frame);

        /*rc = render_hud(ctx, hud.data, width, height);
        if (rc != 0) {
            fprintf(stderr, "can't render HUD: %d\n", rc);
            return -1;
        }*/

        //cv::imshow(window, hud);
        //cv::waitKey(0);
        //break;
    }

    auto end = std::chrono::steady_clock::now();

    std::cout << "TIME: " 
        << std::chrono::duration_cast<std::chrono::milliseconds>(end - start).count()
        << " ms" << std::endl;

    return 0;
}