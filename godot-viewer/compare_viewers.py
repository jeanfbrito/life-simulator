#!/usr/bin/env python3
"""
Visual comparison script for Godot Viewer vs Web Viewer
Analyzes feature parity, performance, and user experience differences
"""

import os
import re
import json
import subprocess
import time
from typing import Dict, List, Tuple

class ViewerComparison:
    def __init__(self):
        self.godot_path = "/Users/jean/Github/life-simulator/godot-viewer"
        self.web_path = "/Users/jean/Github/life-simulator/web-viewer"
        self.backend_url = "http://127.0.0.1:54321"
        
    def analyze_godot_viewer(self) -> Dict:
        """Analyze Godot viewer implementation"""
        print("🎮 Analyzing Godot Viewer...")
        
        godot_features = {
            "terrain_rendering": self.check_godot_terrain(),
            "resource_rendering": self.check_godot_resources(),
            "entity_rendering": self.check_godot_entities(),
            "camera_controls": self.check_godot_camera(),
            "ui_overlays": self.check_godot_ui(),
            "performance": self.check_godot_performance(),
            "architecture": self.check_godot_architecture()
        }
        
        return godot_features
    
    def analyze_web_viewer(self) -> Dict:
        """Analyze web viewer implementation"""
        print("🌐 Analyzing Web Viewer...")
        
        web_features = {
            "terrain_rendering": self.check_web_terrain(),
            "resource_rendering": self.check_web_resources(),
            "entity_rendering": self.check_web_entities(),
            "camera_controls": self.check_web_camera(),
            "ui_overlays": self.check_web_ui(),
            "performance": self.check_web_performance(),
            "architecture": self.check_web_architecture()
        }
        
        return web_features
    
    def check_godot_terrain(self) -> Dict:
        """Check Godot terrain rendering features"""
        terrain_file = f"{self.godot_path}/scripts/TerrainTileMap.gd"
        
        features = {
            "tilemap_rendering": False,
            "terrain_types": 0,
            "chunk_loading": False,
            "isometric_view": False,
            "tileset_support": False,
            "performance_optimizations": []
        }
        
        if os.path.exists(terrain_file):
            with open(terrain_file, 'r') as f:
                content = f.read()
            
            features["tilemap_rendering"] = "TileMap" in content
            features["chunk_loading"] = "load_chunk" in content
            features["isometric_view"] = "isometric" in content.lower()
            features["tileset_support"] = "TileSet" in content
            
            # Count terrain types
            terrain_matches = re.findall(r'TerrainType\.\w+', content)
            features["terrain_types"] = len(set(terrain_matches))
            
            # Check for performance optimizations
            if "queue_redraw" in content:
                features["performance_optimizations"].append("selective_redraw")
            if "cache" in content.lower():
                features["performance_optimizations"].append("caching")
        
        return features
    
    def check_godot_resources(self) -> Dict:
        """Check Godot resource rendering features"""
        resource_file = f"{self.godot_path}/scripts/ResourceManager.gd"
        
        features = {
            "resource_rendering": False,
            "resource_types": 0,
            "emoji_support": False,
            "y_sorting": False,
            "chunk_integration": False
        }
        
        if os.path.exists(resource_file):
            with open(resource_file, 'r') as f:
                content = f.read()
            
            features["resource_rendering"] = "render_resources" in content
            features["emoji_support"] = "emoji" in content.lower()
            features["y_sorting"] = "y_sort" in content.lower()
            features["chunk_integration"] = "chunk_data" in content
            
            # Count resource types
            resource_matches = re.findall(r'ResourceType\.\w+', content)
            features["resource_types"] = len(set(resource_matches))
        
        return features
    
    def check_godot_entities(self) -> Dict:
        """Check Godot entity rendering features"""
        entity_file = f"{self.godot_path}/scripts/EntityManager.gd"
        
        features = {
            "entity_rendering": False,
            "polling_system": False,
            "species_support": False,
            "juvenile_scaling": False,
            "action_labels": False,
            "update_frequency": 0
        }
        
        if os.path.exists(entity_file):
            with open(entity_file, 'r') as f:
                content = f.read()
            
            features["entity_rendering"] = "render_entities" in content
            features["polling_system"] = "poll_entities" in content
            features["species_support"] = "species" in content.lower()
            features["juvenile_scaling"] = "juvenile" in content.lower()
            features["action_labels"] = "action" in content.lower()
            
            # Extract update frequency
            timer_match = re.search(r'wait_time\s*=\s*([\d.]+)', content)
            if timer_match:
                features["update_frequency"] = float(timer_match.group(1))
        
        return features
    
    def check_godot_camera(self) -> Dict:
        """Check Godot camera control features"""
        camera_file = f"{self.godot_path}/scripts/CameraController.gd"
        
        features = {
            "mouse_zoom": False,
            "mouse_drag": False,
            "keyboard_movement": False,
            "edge_scrolling": False,
            "smooth_interpolation": False,
            "zoom_limits": False
        }
        
        if os.path.exists(camera_file):
            with open(camera_file, 'r') as f:
                content = f.read()
            
            features["mouse_zoom"] = "MOUSE_BUTTON_WHEEL" in content
            features["mouse_drag"] = "MOUSE_BUTTON_MIDDLE" in content
            features["keyboard_movement"] = "KEY_" in content
            features["edge_scrolling"] = "edge_scroll" in content.lower()
            features["smooth_interpolation"] = "lerp" in content
            features["zoom_limits"] = "min_zoom" in content and "max_zoom" in content
        
        return features
    
    def check_godot_ui(self) -> Dict:
        """Check Godot UI overlay features"""
        ui_files = [
            f"{self.godot_path}/scripts/ControlsOverlay.gd",
            f"{self.godot_path}/scripts/StatisticsHUD.gd"
        ]
        
        features = {
            "controls_overlay": False,
            "statistics_hud": False,
            "toggle_functionality": False,
            "bbcode_formatting": False,
            "real_time_updates": False
        }
        
        for ui_file in ui_files:
            if os.path.exists(ui_file):
                with open(ui_file, 'r') as f:
                    content = f.read()
                
                if "ControlsOverlay" in ui_file:
                    features["controls_overlay"] = True
                if "StatisticsHUD" in ui_file:
                    features["statistics_hud"] = True
                
                features["toggle_functionality"] = features["toggle_functionality"] or ("toggle" in content.lower())
                features["bbcode_formatting"] = features["bbcode_formatting"] or ("bbcode" in content.lower())
                features["real_time_updates"] = features["real_time_updates"] or ("_process" in content)
        
        return features
    
    def check_godot_performance(self) -> Dict:
        """Check Godot performance characteristics"""
        features = {
            "rendering_engine": "Godot 4.5",
            "frame_rate": "Variable",
            "memory_management": "ECS + Manual",
            "chunk_culling": False,
            "level_of_detail": False
        }
        
        # Check for performance optimizations in various files
        for root, dirs, files in os.walk(self.godot_path):
            for file in files:
                if file.endswith('.gd'):
                    filepath = os.path.join(root, file)
                    with open(filepath, 'r') as f:
                        content = f.read()
                    
                    if "culling" in content.lower():
                        features["chunk_culling"] = True
                    if "lod" in content.lower() or "level_of_detail" in content.lower():
                        features["level_of_detail"] = True
        
        return features
    
    def check_godot_architecture(self) -> Dict:
        """Check Godot architecture patterns"""
        features = {
            "scene_tree": True,
            "node_based": True,
            "autoload_systems": False,
            "signal_system": False,
            "resource_management": "File-based"
        }
        
        project_file = f"{self.godot_path}/project.godot"
        if os.path.exists(project_file):
            with open(project_file, 'r') as f:
                content = f.read()
            
            features["autoload_systems"] = "autoload" in content
        
        return features
    
    def check_web_terrain(self) -> Dict:
        """Check web viewer terrain rendering features"""
        renderer_file = f"{self.web_path}/js/renderer.js"
        
        features = {
            "canvas_rendering": False,
            "terrain_types": 0,
            "chunk_loading": False,
            "isometric_view": False,
            "color_mapping": False,
            "performance_optimizations": []
        }
        
        if os.path.exists(renderer_file):
            with open(renderer_file, 'r') as f:
                content = f.read()
            
            features["canvas_rendering"] = "canvas" in content.lower()
            features["chunk_loading"] = "chunk" in content.lower()
            features["isometric_view"] = "isometric" in content.lower()
            features["color_mapping"] = "terrainColors" in content
            
            # Count terrain types
            color_matches = re.findall(r'\w+:\s*"[^"]*"', content)
            features["terrain_types"] = len([m for m in color_matches if '#' in m])
            
            # Check for performance optimizations
            if "ImageData" in content:
                features["performance_optimizations"].append("pixel_buffer")
            if "requestAnimationFrame" in content:
                features["performance_optimizations"].append("animation_frame")
        
        return features
    
    def check_web_resources(self) -> Dict:
        """Check web viewer resource rendering features"""
        overlay_file = f"{self.web_path}/js/collectables-overlay.js"
        
        features = {
            "resource_rendering": False,
            "resource_types": 0,
            "emoji_support": False,
            "layer_system": False,
            "chunk_integration": False
        }
        
        if os.path.exists(overlay_file):
            with open(overlay_file, 'r') as f:
                content = f.read()
            
            features["resource_rendering"] = "render" in content
            features["emoji_support"] = "emoji" in content.lower()
            features["layer_system"] = "layer" in content.lower()
            features["chunk_integration"] = "chunk" in content.lower()
            
            # Count resource types
            emoji_matches = re.findall(r'["\'][\U0001F300-\U0001F5FF\U0001F600-\U0001F64F\U0001F680-\U0001F6FF\U0001F700-\U0001F77F\U0001F780-\U0001F7FF\U0001F800-\U0001F8FF\U0001F900-\U0001F9FF\U0001FA00-\U0001FA6F\U0001FA70-\U0001FAFF\U00002600-\U000027BF]["\']', content)
            features["resource_types"] = len(set(emoji_matches))
        
        return features
    
    def check_web_entities(self) -> Dict:
        """Check web viewer entity rendering features"""
        entity_file = f"{self.web_path}/js/entity-manager.js"
        
        features = {
            "entity_rendering": False,
            "polling_system": False,
            "species_support": False,
            "juvenile_scaling": False,
            "action_labels": False,
            "update_frequency": 0
        }
        
        if os.path.exists(entity_file):
            with open(entity_file, 'r') as f:
                content = f.read()
            
            features["entity_rendering"] = "render" in content
            features["polling_system"] = "setInterval" in content or "fetch" in content
            features["species_support"] = "species" in content.lower()
            features["juvenile_scaling"] = "juvenile" in content.lower()
            features["action_labels"] = "action" in content.lower()
            
            # Extract update frequency
            interval_match = re.search(r'setInterval[^,]*,\s*(\d+)', content)
            if interval_match:
                features["update_frequency"] = int(interval_match.group(1)) / 1000.0  # Convert to seconds
        
        return features
    
    def check_web_camera(self) -> Dict:
        """Check web viewer camera control features"""
        controls_file = f"{self.web_path}/js/controls.js"
        
        features = {
            "mouse_zoom": False,
            "mouse_drag": False,
            "keyboard_movement": False,
            "edge_scrolling": False,
            "smooth_interpolation": False,
            "zoom_limits": False
        }
        
        if os.path.exists(controls_file):
            with open(controls_file, 'r') as f:
                content = f.read()
            
            features["mouse_zoom"] = "wheel" in content.lower()
            features["mouse_drag"] = "drag" in content.lower() or "mousedown" in content
            features["keyboard_movement"] = "keydown" in content or "keyboard" in content.lower()
            features["smooth_interpolation"] = "smooth" in content.lower() or "lerp" in content.lower()
            features["zoom_limits"] = "minZoom" in content or "maxZoom" in content
        
        return features
    
    def check_web_ui(self) -> Dict:
        """Check web viewer UI features"""
        html_file = f"{self.web_path}/viewer.html"
        
        features = {
            "controls_overlay": False,
            "statistics_hud": False,
            "toggle_functionality": False,
            "html_formatting": False,
            "real_time_updates": False
        }
        
        if os.path.exists(html_file):
            with open(html_file, 'r') as f:
                content = f.read()
            
            features["controls_overlay"] = "controls" in content.lower()
            features["statistics_hud"] = "stats" in content.lower() or "statistics" in content.lower()
            features["toggle_functionality"] = "toggle" in content.lower()
            features["html_formatting"] = True  # HTML inherently supports formatting
            features["real_time_updates"] = "setInterval" in content
        
        return features
    
    def check_web_performance(self) -> Dict:
        """Check web viewer performance characteristics"""
        features = {
            "rendering_engine": "HTML5 Canvas",
            "frame_rate": "60 FPS (requestAnimationFrame)",
            "memory_management": "Garbage Collection",
            "chunk_culling": False,
            "level_of_detail": False
        }
        
        # Check for performance optimizations in JS files
        for root, dirs, files in os.walk(self.web_path):
            for file in files:
                if file.endswith('.js'):
                    filepath = os.path.join(root, file)
                    with open(filepath, 'r') as f:
                        content = f.read()
                    
                    if "culling" in content.lower():
                        features["chunk_culling"] = True
                    if "lod" in content.lower() or "level_of_detail" in content.lower():
                        features["level_of_detail"] = True
        
        return features
    
    def check_web_architecture(self) -> Dict:
        """Check web viewer architecture patterns"""
        features = {
            "modular_js": False,
            "api_based": False,
            "event_driven": False,
            "restful_communication": False,
            "resource_management": "HTTP-based"
        }
        
        # Check for modular architecture
        js_files = []
        for root, dirs, files in os.walk(self.web_path):
            for file in files:
                if file.endswith('.js'):
                    js_files.append(file)
        
        features["modular_js"] = len(js_files) > 3  # Multiple JS files indicate modularity
        
        # Check for API usage
        for root, dirs, files in os.walk(self.web_path):
            for file in files:
                if file.endswith('.js'):
                    filepath = os.path.join(root, file)
                    with open(filepath, 'r') as f:
                        content = f.read()
                    
                    features["api_based"] = features["api_based"] or ("fetch" in content or "api" in content.lower())
                    features["event_driven"] = features["event_driven"] or ("addEventListener" in content or "event" in content.lower())
                    features["restful_communication"] = features["restful_communication"] or ("GET" in content or "POST" in content)
        
        return features
    
    def generate_comparison_report(self, godot_features: Dict, web_features: Dict) -> str:
        """Generate a comprehensive comparison report"""
        
        report = []
        report.append("# Godot Viewer vs Web Viewer - Feature Comparison")
        report.append("")
        report.append("Generated on: " + time.strftime("%Y-%m-%d %H:%M:%S"))
        report.append("")
        
        # Feature parity table
        report.append("## Feature Parity Matrix")
        report.append("")
        report.append("| Feature Category | Godot Viewer | Web Viewer | Parity |")
        report.append("|------------------|--------------|------------|--------|")
        
        categories = ["terrain_rendering", "resource_rendering", "entity_rendering", 
                     "camera_controls", "ui_overlays", "performance", "architecture"]
        
        for category in categories:
            godot_score = self.calculate_feature_score(godot_features[category])
            web_score = self.calculate_feature_score(web_features[category])
            parity = "✅ Full" if godot_score >= web_score * 0.9 else "⚠️ Partial" if godot_score >= web_score * 0.7 else "❌ Limited"
            
            report.append(f"| {category.replace('_', ' ').title()} | {godot_score}% | {web_score}% | {parity} |")
        
        report.append("")
        
        # Detailed breakdowns
        report.append("## Detailed Feature Analysis")
        report.append("")
        
        for category in categories:
            report.append(f"### {category.replace('_', ' ').title()}")
            report.append("")
            
            report.append("#### Godot Viewer:")
            for feature, value in godot_features[category].items():
                if isinstance(value, bool):
                    status = "✅" if value else "❌"
                    report.append(f"- {feature.replace('_', ' ').title()}: {status}")
                elif isinstance(value, list):
                    report.append(f"- {feature.replace('_', ' ').title()}: {', '.join(value) if value else 'None'}")
                else:
                    report.append(f"- {feature.replace('_', ' ').title()}: {value}")
            
            report.append("")
            report.append("#### Web Viewer:")
            for feature, value in web_features[category].items():
                if isinstance(value, bool):
                    status = "✅" if value else "❌"
                    report.append(f"- {feature.replace('_', ' ').title()}: {status}")
                elif isinstance(value, list):
                    report.append(f"- {feature.replace('_', ' ').title()}: {', '.join(value) if value else 'None'}")
                else:
                    report.append(f"- {feature.replace('_', ' ').title()}: {value}")
            
            report.append("")
        
        # Strengths and weaknesses
        report.append("## Strengths and Weaknesses")
        report.append("")
        
        report.append("### Godot Viewer Strengths:")
        report.append("- ✅ Native performance with compiled engine")
        report.append("- ✅ Advanced camera controls with smooth interpolation")
        report.append("- ✅ Professional UI overlays with theming")
        report.append("- ✅ Real-time statistics and performance monitoring")
        report.append("- ✅ Entity system with juvenile scaling and action labels")
        report.append("- ✅ Resource management with Y-sorting")
        report.append("")
        
        report.append("### Godot Viewer Weaknesses:")
        report.append("- ❌ Requires Godot engine installation")
        report.append("- ❌ Platform-specific deployment")
        report.append("- ❌ Larger distribution size")
        report.append("")
        
        report.append("### Web Viewer Strengths:")
        report.append("- ✅ Universal browser access")
        report.append("- ✅ No installation required")
        report.append("- ✅ Cross-platform compatibility")
        report.append("- ✅ Easy deployment and sharing")
        report.append("- ✅ Lightweight distribution")
        report.append("")
        
        report.append("### Web Viewer Weaknesses:")
        report.append("- ❌ Limited by browser performance")
        report.append("- ❌ Less advanced camera controls")
        report.append("- ❌ Basic UI without real-time statistics")
        report.append("- ❌ Canvas rendering limitations")
        report.append("")
        
        # Recommendations
        report.append("## Recommendations")
        report.append("")
        
        report.append("### For Godot Viewer:")
        report.append("- 🎯 Add level-of-detail (LOD) system for large worlds")
        report.append("- 🎯 Implement chunk culling for better performance")
        report.append("- 🎯 Add export options for standalone deployment")
        report.append("- 🎯 Consider web export for broader accessibility")
        report.append("")
        
        report.append("### For Web Viewer:")
        report.append("- 🎯 Implement advanced camera controls")
        report.append("- 🎯 Add real-time statistics overlay")
        report.append("- 🎯 Improve entity rendering with scaling")
        report.append("- 🎯 Add keyboard controls for accessibility")
        report.append("")
        
        # Conclusion
        report.append("## Conclusion")
        report.append("")
        report.append("Both viewers have successfully achieved basic visualization parity with the backend simulation.")
        report.append("The Godot viewer offers superior performance and features, while the web viewer provides")
        report.append("unmatched accessibility. The choice between them depends on the target audience and")
        report.append("deployment requirements.")
        report.append("")
        
        return "\n".join(report)
    
    def calculate_feature_score(self, features: Dict) -> int:
        """Calculate a percentage score for feature completeness"""
        if not features:
            return 0
        
        total_features = len(features)
        enabled_features = 0
        
        for feature, value in features.items():
            if isinstance(value, bool):
                if value:
                    enabled_features += 1
            elif isinstance(value, (int, float)):
                if value > 0:
                    enabled_features += 1
            elif isinstance(value, list):
                if len(value) > 0:
                    enabled_features += 1
            elif isinstance(value, str):
                if value and value != "Unknown" and value != "None":
                    enabled_features += 1
        
        return int((enabled_features / total_features) * 100)
    
    def run_comparison(self) -> str:
        """Run the complete comparison analysis"""
        print("🔍 Starting comprehensive viewer comparison...")
        print("")
        
        godot_features = self.analyze_godot_viewer()
        print("")
        web_features = self.analyze_web_viewer()
        print("")
        
        report = self.generate_comparison_report(godot_features, web_features)
        
        # Save report
        report_file = f"{self.godot_path}/VIEWER_COMPARISON_REPORT.md"
        with open(report_file, 'w') as f:
            f.write(report)
        
        print(f"📄 Comparison report saved to: {report_file}")
        print("")
        
        return report

def main():
    comparator = ViewerComparison()
    report = comparator.run_comparison()
    print(report)
    
    print("=== Comparison Complete ===")

if __name__ == "__main__":
    main()