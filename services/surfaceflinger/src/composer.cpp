#include "composer.h"
#include <iostream>
#include <algorithm>
#include <sys/stat.h>
#include <sys/types.h>

Composer::Composer() : mNextSurfaceId(1) {
    std::cout << "[INFO] [SURFACEFLINGER] Graphics Composer engine initialized." << std::endl;
}

Composer::~Composer() {
    mSurfaces.clear();
}

int32_t Composer::allocateSurface(int32_t width, int32_t height, int32_t format) {
    Surface s;
    s.id = mNextSurfaceId++;
    s.width = width;
    s.height = height;
    s.format = format;
    s.active = true;

    mSurfaces.push_back(s);
    std::cout << "[INFO] [SURFACEFLINGER] Allocated surface ID " << s.id 
              << " (" << s.width << "x" << s.height << ", Format: " << s.format << ")" << std::endl;
    return s.id;
}

int Composer::destroySurface(int32_t id) {
    auto it = std::find_if(mSurfaces.begin(), mSurfaces.end(), [id](const Surface &s) {
        return s.id == id;
    });

    if (it != mSurfaces.end()) {
        std::cout << "[INFO] [SURFACEFLINGER] Destroyed surface ID " << id << std::endl;
        mSurfaces.erase(it);
        return 0;
    }

    std::cerr << "[WARN] [SURFACEFLINGER] Attempted to destroy non-existent surface ID " << id << std::endl;
    return -1;
}

#include <fstream>
#include <sstream>
#include <algorithm>

struct PPMImage {
    int width = 0;
    int height = 0;
    std::vector<uint8_t> rgb_data;
};

static PPMImage load_ppm(const std::string &path) {
    PPMImage img;
    std::ifstream f(path, std::ios::binary);
    if (!f.is_open()) {
        return img;
    }
    
    std::string format;
    f >> format;
    if (format != "P6") return img;
    
    f >> std::ws;
    while (f.peek() == '#') {
        std::string comment;
        std::getline(f, comment);
        f >> std::ws;
    }
    
    f >> img.width >> img.height;
    int max_val;
    f >> max_val;
    
    char ch;
    f.get(ch); // Skip single whitespace character
    
    size_t data_size = img.width * img.height * 3;
    img.rgb_data.resize(data_size);
    f.read(reinterpret_cast<char*>(img.rgb_data.data()), data_size);
    
    return img;
}

int Composer::compositeLayers() {
    if (mSurfaces.empty()) {
        return 0;
    }

    std::cout << "[INFO] [SURFACEFLINGER] Starting software frame composition..." << std::endl;
    
    // Master RGB Buffer representing 1080 x 2400 phone screen
    const int master_w = 1080;
    const int master_h = 2400;
    std::vector<uint8_t> master_rgb(master_w * master_h * 3, 0);
    
    // Fill background with a deep dark slate color
    for (int i = 0; i < master_w * master_h; i++) {
        master_rgb[i * 3 + 0] = 15;  // R
        master_rgb[i * 3 + 1] = 13;  // G
        master_rgb[i * 3 + 2] = 22;  // B
    }

    // Sort layers by height descending so smaller overlays (statusbar) are rendered last on top
    std::vector<Surface> sorted_surfaces = mSurfaces;
    std::sort(sorted_surfaces.begin(), sorted_surfaces.end(), [](const Surface &a, const Surface &b) {
        return a.height > b.height;
    });

    for (const auto &s : sorted_surfaces) {
        if (!s.active) continue;

        std::string file_path = "out/surface_" + std::to_string(s.id) + ".ppm";
        PPMImage img = load_ppm(file_path);
        
        if (img.rgb_data.empty()) {
            std::cout << "  ⚠️  No visual buffer found for surface ID " << s.id 
                      << " (looked for " << file_path << ")" << std::endl;
            continue;
        }

        // Determine starting y offset based on layer heights
        int y_start = 0;
        if (s.height == 2200) {
            y_start = 100; // middle application window
        } else if (s.height <= 100) {
            y_start = 0;   // top statusbar panel overlay
        }

        std::cout << "  👉 Compositing surface layer ID " << s.id 
                  << " [" << s.width << "x" << s.height << "] at y=" << y_start << std::endl;

        // Copy source pixels to master canvas
        for (int src_y = 0; src_y < img.height; src_y++) {
            int dest_y = y_start + src_y;
            if (dest_y >= master_h) break;

            for (int src_x = 0; src_x < img.width; src_x++) {
                int dest_x = src_x;
                if (dest_x >= master_w) break;

                size_t src_idx = (src_y * img.width + src_x) * 3;
                size_t dest_idx = (dest_y * master_w + dest_x) * 3;

                master_rgb[dest_idx + 0] = img.rgb_data[src_idx + 0];
                master_rgb[dest_idx + 1] = img.rgb_data[src_idx + 1];
                master_rgb[dest_idx + 2] = img.rgb_data[src_idx + 2];
            }
        }
    }

    // Save final combined PPM image
    mkdir("out", 0777);
    std::ofstream out("out/display_composited.ppm", std::ios::binary);
    if (out.is_open()) {
        out << "P6\n" << master_w << " " << master_h << "\n255\n";
        out.write(reinterpret_cast<const char*>(master_rgb.data()), master_rgb.size());
        std::cout << "[INFO] [SURFACEFLINGER] Saved combined composited frame to out/display_composited.ppm" << std::endl;
    } else {
        std::cerr << "[ERR] [SURFACEFLINGER] Failed to write to out/display_composited.ppm" << std::endl;
    }

    return 0;
}

