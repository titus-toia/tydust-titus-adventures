#!/usr/bin/env python3
"""
Analyze isometric asset orientation using computer vision
Detects edges, dominant angles, and perspective tilt
"""

import sys
import cv2
import numpy as np
from PIL import Image
import matplotlib.pyplot as plt

try:
    from skimage.measure import regionprops
    from skimage.morphology import binary_erosion, binary_dilation
    SKIMAGE_AVAILABLE = True
except ImportError:
    SKIMAGE_AVAILABLE = False
    print("⚠ scikit-image not available, skipping region-based orientation detection")

def detect_orientation_via_pca(alpha_mask):
    """Detect orientation using PCA (Principal Component Analysis) on pixel distribution

    This is the 'heatmap' approach - it analyzes the distribution of all pixels,
    not just edges, to find the dominant orientation axes.
    """
    # Get coordinates of all non-transparent pixels
    y_coords, x_coords = np.where(alpha_mask > 10)

    if len(x_coords) < 100:
        return None

    # Stack into coordinate matrix
    coords = np.column_stack([x_coords, y_coords])

    # Center the coordinates
    mean = coords.mean(axis=0)
    centered = coords - mean

    # Compute covariance matrix
    cov = np.cov(centered.T)

    # Compute eigenvalues and eigenvectors
    eigenvalues, eigenvectors = np.linalg.eig(cov)

    # The eigenvector with largest eigenvalue is the principal axis (dominant orientation)
    idx = eigenvalues.argsort()[::-1]
    principal_axis = eigenvectors[:, idx[0]]

    # Calculate angle from horizontal
    angle = np.degrees(np.arctan2(principal_axis[1], principal_axis[0]))

    # Normalize to -90 to +90
    if angle > 90:
        angle -= 180
    elif angle < -90:
        angle += 180

    return {
        'angle': angle,
        'eigenvalues': eigenvalues[idx],
        'eigenvectors': eigenvectors[:, idx],
        'mean': mean,
    }

def detect_orientation_via_regionprops(alpha_mask):
    """Detect orientation using scikit-image regionprops (moment-based)"""
    if not SKIMAGE_AVAILABLE:
        return None

    # Convert to binary
    binary = (alpha_mask > 10).astype(np.uint8)

    # Get region properties
    props = regionprops(binary)

    if len(props) == 0:
        return None

    # Get the largest region (should be the whole sprite)
    region = props[0]

    # Orientation in radians (-pi/2 to pi/2)
    orientation_rad = region.orientation
    orientation_deg = np.degrees(orientation_rad)

    return {
        'angle': orientation_deg,
        'centroid': region.centroid,
        'major_axis_length': region.major_axis_length,
        'minor_axis_length': region.minor_axis_length,
        'eccentricity': region.eccentricity,
    }

def analyze_orientation(image_path):
    """Detect the orientation and perspective of an isometric asset"""

    # Load image
    img = Image.open(image_path).convert('RGBA')

    # Convert to numpy array and extract RGB (OpenCV uses BGR)
    data = np.array(img)
    rgb = cv2.cvtColor(data[:, :, :3], cv2.COLOR_RGB2BGR)
    alpha = data[:, :, 3]

    # Create mask of non-transparent pixels
    mask = (alpha > 10).astype(np.uint8) * 255

    # Apply mask to get only the platform content
    masked = cv2.bitwise_and(rgb, rgb, mask=mask)
    gray = cv2.cvtColor(masked, cv2.COLOR_BGR2GRAY)

    print(f"Image size: {img.size[0]}×{img.size[1]}")
    print(f"Non-transparent pixels: {np.sum(alpha > 10)}")

    # Find edges
    edges = cv2.Canny(gray, 50, 150, apertureSize=3)

    # Detect lines using Hough transform
    lines = cv2.HoughLinesP(edges, 1, np.pi/180, threshold=100,
                            minLineLength=100, maxLineGap=20)

    if lines is None:
        print("✗ No lines detected")
        return None

    print(f"Detected {len(lines)} lines")

    # Calculate angles of all lines
    angles = []
    for line in lines:
        x1, y1, x2, y2 = line[0]
        angle = np.degrees(np.arctan2(y2 - y1, x2 - x1))
        # Normalize to -90 to +90 range
        if angle > 90:
            angle -= 180
        elif angle < -90:
            angle += 180
        angles.append(angle)

    angles = np.array(angles)

    # Find dominant angles (cluster around common orientations)
    # Filter out nearly horizontal/vertical lines (likely edges)
    angled_lines = angles[(np.abs(angles) > 5) & (np.abs(angles) < 85)]

    if len(angled_lines) == 0:
        print("✗ No angled lines detected")
        return None

    # Get dominant angle via histogram
    hist, bins = np.histogram(angled_lines, bins=36, range=(-90, 90))
    dominant_bin = np.argmax(hist)
    dominant_angle = (bins[dominant_bin] + bins[dominant_bin + 1]) / 2

    # Also try PCA-based orientation detection (pixel distribution heatmap)
    pca_result = detect_orientation_via_pca(alpha)
    regionprops_result = detect_orientation_via_regionprops(alpha)

    print(f"\n=== ORIENTATION ANALYSIS (Multiple Methods) ===")
    print(f"Hough lines dominant angle: {dominant_angle:.1f}°")
    print(f"  └─ Detects edge orientations")

    if pca_result:
        print(f"PCA pixel distribution angle: {pca_result['angle']:.1f}°")
        print(f"  └─ Finds principal axis of all pixels (heatmap method)")
        print(f"  └─ Eigenvalues: [{pca_result['eigenvalues'][0]:.0f}, {pca_result['eigenvalues'][1]:.0f}]")

    if regionprops_result:
        print(f"RegionProps moment angle: {regionprops_result['angle']:.1f}°")
        print(f"  └─ Calculates from 2nd order moments")
        print(f"  └─ Major/Minor axes: {regionprops_result['major_axis_length']:.0f} / {regionprops_result['minor_axis_length']:.0f}")

    print(f"\nLine angles (sample): {angled_lines[:10]}")

    # Detect perspective tilt by analyzing vertical edges
    vertical_ish = angles[(np.abs(angles) > 60) & (np.abs(angles) < 90)]
    if len(vertical_ish) > 0:
        avg_vertical = np.median(vertical_ish)
        vertical_tilt = 90 - np.abs(avg_vertical)
        print(f"Vertical edge tilt: {vertical_tilt:.1f}° from true vertical")

    # Find bounding box of content
    rows = np.any(alpha > 10, axis=1)
    cols = np.any(alpha > 10, axis=0)
    y_min, y_max = np.where(rows)[0][[0, -1]]
    x_min, x_max = np.where(cols)[0][[0, -1]]

    # Calculate center and dimensions
    center_x = (x_min + x_max) / 2
    center_y = (y_min + y_max) / 2
    width = x_max - x_min
    height = y_max - y_min

    print(f"\nContent bounds: {width:.0f}×{height:.0f}px")
    print(f"Center: ({center_x:.0f}, {center_y:.0f})")

    # Visualize
    vis = rgb.copy()

    # Draw detected lines
    if lines is not None:
        for line in lines[:50]:  # Draw first 50 lines
            x1, y1, x2, y2 = line[0]
            angle = np.degrees(np.arctan2(y2 - y1, x2 - x1))
            # Color code by angle
            if -90 <= angle <= 90:
                if np.abs(angle) < 5:
                    color = (0, 255, 0)  # Green = horizontal
                elif np.abs(angle - 90) < 5 or np.abs(angle + 90) < 5:
                    color = (255, 0, 0)  # Blue = vertical
                else:
                    color = (0, 165, 255)  # Orange = angled
                cv2.line(vis, (x1, y1), (x2, y2), color, 2)

    # Draw bounding box
    cv2.rectangle(vis, (x_min, y_min), (x_max, y_max), (255, 255, 0), 2)

    # Draw center
    cv2.circle(vis, (int(center_x), int(center_y)), 5, (0, 0, 255), -1)

    # Show visualization
    vis_rgb = cv2.cvtColor(vis, cv2.COLOR_BGR2RGB)
    plt.figure(figsize=(12, 10))
    plt.imshow(vis_rgb)
    plt.title(f"Orientation Analysis: {dominant_angle:.1f}° dominant angle")
    plt.axis('off')
    plt.tight_layout()
    plt.savefig('orientation_analysis.png', dpi=150, bbox_inches='tight')
    print(f"\n✓ Saved visualization to: orientation_analysis.png")

    # Draw principal axes from PCA if available
    if pca_result:
        mean_x, mean_y = pca_result['mean']
        for i, eigvec in enumerate(pca_result['eigenvectors'].T):
            length = np.sqrt(pca_result['eigenvalues'][i]) * 0.5
            end_x = int(mean_x + eigvec[0] * length)
            end_y = int(mean_y + eigvec[1] * length)
            color = (255, 0, 255) if i == 0 else (0, 255, 255)  # Magenta for primary, cyan for secondary
            cv2.arrowedLine(vis, (int(mean_x), int(mean_y)), (end_x, end_y), color, 3, tipLength=0.3)

    return {
        'hough_angle': dominant_angle,
        'pca_angle': pca_result['angle'] if pca_result else None,
        'regionprops_angle': regionprops_result['angle'] if regionprops_result else None,
        'width': width,
        'height': height,
        'center': (center_x, center_y),
        'bounds': (x_min, y_min, x_max, y_max),
    }

def calculate_isometric_square_positions(asset_info, spacing_multiplier=0.7):
    """Calculate positions for 4 platforms in isometric square formation

    Uses the detected orientation to place platforms in proper 3D arrangement
    """
    # Use PCA angle if available (most reliable for overall shape orientation),
    # otherwise fall back to Hough lines
    angle = asset_info.get('pca_angle') or asset_info.get('hough_angle')
    width = asset_info['width']
    height = asset_info['height']

    print(f"\n=== ISOMETRIC SQUARE CALCULATION ===")
    print(f"Using angle: {angle:.1f}° (PCA)" if asset_info.get('pca_angle') else f"Using angle: {angle:.1f}° (Hough)")
    print(f"Asset dimensions: {width:.0f}×{height:.0f}")

    # For isometric square, we need to calculate spacing along the platform's axes
    # If platform is tilted ~15° right, that's the "receding" axis

    # Calculate spacing along the tilted axis
    angle_rad = np.radians(angle)

    # X and Y components of movement along the platform's orientation
    dx = np.cos(angle_rad) * width * spacing_multiplier
    dy = np.sin(angle_rad) * width * spacing_multiplier

    print(f"Spacing vector: ΔX={dx:.1f}, ΔY={dy:.1f}")

    # Four corners of isometric square
    # In isometric view: back-left, back-right, front-left, front-right
    positions = {
        'back_left': {'x': -dx/2, 'y_offset': -dy/2, 'z_order': 1},
        'back_right': {'x': dx/2, 'y_offset': -dy/2, 'z_order': 2},
        'front_left': {'x': -dx/2, 'y_offset': dy/2, 'z_order': 3},
        'front_right': {'x': dx/2, 'y_offset': dy/2, 'z_order': 4},
    }

    return positions

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python analyze_asset_orientation.py <image_path>")
        sys.exit(1)

    image_path = sys.argv[1]

    info = analyze_orientation(image_path)

    if info:
        positions = calculate_isometric_square_positions(info)

        print("\n=== SUGGESTED POSITIONS ===")
        for name, pos in positions.items():
            print(f"{name}: X={pos['x']:.0f}, Y_offset={pos['y_offset']:.0f}, z_order={pos['z_order']}")
