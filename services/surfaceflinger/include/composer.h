#pragma once
#include <stdint.h>
#include <vector>

// Window layer properties representing client applications
struct Surface {
    int32_t id;
    int32_t width;
    int32_t height;
    int32_t format;
    bool active;
};

// Graphics Composer engine managing display composition
class Composer {
public:
    Composer();
    ~Composer();

    int32_t allocateSurface(int32_t width, int32_t height, int32_t format);
    int destroySurface(int32_t id);
    int compositeLayers();

private:
    std::vector<Surface> mSurfaces;
    int32_t mNextSurfaceId;
};
