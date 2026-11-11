//! Native App Hub. The artwork is the only raster part of these screens.
use crate::catalog::{
    self, Backend, CatalogKind, CatalogSnapshot, Entry, EntryStatus, InstallConsent, ToolUse,
};
use makepad_widgets::image_cache::handle_image_cache_network_responses;
use makepad_widgets::makepad_platform::thread::{SignalToUI, ThreadOptions};
use makepad_widgets::*;
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};

fn skin(vm: &mut ScriptVm, light: u32, dark: u32) -> Vec4f {
    Vec4f::from_u32(
        if makepad_wm_theme::current_for_vm(vm).is_none_or(|p| p.light_mode) {
            light
        } else {
            dark
        },
    )
}

/// A desktop: the Mac's App Store. A phone keeps its tab bar and touch sizes.
fn is_desk(vm: &mut ScriptVm) -> bool {
    !makepad_widgets::desktop_style::current_style(vm).mobile()
}

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*
    let ground = #(skin(vm, 0xf3f5f9ff, 0x101722ff))
    let card = #(skin(vm, 0xffffffff, 0x1b2636ff))
    let ink = #(skin(vm, 0x111b2bff, 0xf5f7fbff))
    let secondary = #(skin(vm, 0x53647bff, 0xadb9cdff))
    let field = #(skin(vm, 0xe9eef6ff, 0x2c3545ff))
    let accent = #(skin(vm, 0x285cd6ff, 0x96b7ffff))
    let clear = #00000000
    let Heavy = theme.font_bold{font_family +: {latin := FontMember{res: crate_resource("makepad_widgets:resources/Inter.ttf") weight: 800.0 asc: 0.0 desc: 0.0}}}
    let Text = Label{width: Fill padding: 0 draw_text +: {color: ink max_lines: 0 text_overflow: TextOverflow.Clip text_style: theme.font_regular{font_size: 14 line_spacing: 1.3}}}
    let Heading = Text{draw_text.text_style: Heavy{font_size: 22}}
    let Meta = Text{draw_text +: {color: secondary text_style: theme.font_regular{font_size: 13 line_spacing: 1.4}}}
    let Plain = ButtonFlat{height: 44 padding: Inset{left: 10 right: 10} margin: 0 text: "" align: Center
        draw_bg +: {border_size: uniform(0.0) color: uniform(clear) color_hover: uniform(clear) color_down: uniform(#285cd620) color_focus: uniform(clear)
            border_color: uniform(clear) border_color_hover: uniform(clear) border_color_down: uniform(clear) border_color_focus: uniform(clear)}
        draw_text +: {color: accent color_hover: accent color_down: accent color_focus: accent text_style: theme.font_bold{font_size: 13}}
    }
    let Pill = Plain{width: Fit height: 44 padding: Inset{left: 16 right: 16} draw_bg +: {border_radius: uniform(18.0) color: uniform(field) color_hover: uniform(field) color_focus: uniform(field)}}
    let Primary = Plain{width: Fill height: 46 draw_bg +: {border_radius: uniform(14.0) color: uniform(#285cd6) color_hover: uniform(#3468df) color_down: uniform(#204cb8) color_focus: uniform(#204cb8)}
        draw_text +: {color: #fff color_hover: #fff color_down: #fff color_focus: #fff}}
    let Card = RoundedAllView{width: Fill height: Fit flow: Down padding: 16 spacing: 10 draw_bg +: {color: card border_radius: vec4(16.0)}}
    let Item = View{width: Fill height: Fit flow: Down padding: Inset{left: 24 right: 24 bottom: 12}}
    let AppMark = RoundedAllView{width: 54 height: 54 flow: Overlay align: Center draw_bg +: {color: #078be8 border_radius: vec4(12.0)}
        mark := View{width: Fit height: Fit Icon{icon_walk: Walk{width: 28 height: 28} draw_icon +: {svg: crate_resource("self:resources/icons/apps.svg") color: #fff}}}
        builtin := View{visible: false width: Fill height: Fill
            art := AppIcon{width: Fill height: Fill}
        }
        image := Image{visible: false width: Fill height: Fill fit: ImageFit.Smallest}
    }
    let AppRow = Item{
        row := Card{flow: Right padding: Inset{left: 16 right: 16 top: 16 bottom: 16} spacing: 16 align: Align{y: 0.5} cursor: MouseCursor.Hand
            icon := AppMark{}
            View{width: Fill height: Fit flow: Down spacing: 4
                name := Text{max_lines: 1 text_overflow: Ellipsis draw_text.text_style: theme.font_bold{font_size: 17}}
                subtitle := Meta{max_lines: 2 text_overflow: Ellipsis}
            }
            action := Pill{text: "Get"}
        }
    }
    let Tab = Plain{width: Fill height: 58 flow: Down spacing: 4 padding: 4 icon_walk: Walk{width: 22 height: 22}
        draw_icon.color: secondary draw_text +: {color: secondary color_hover: accent color_down: accent color_focus: accent text_style: theme.font_bold{font_size: 12}}}

    let PhoneFrame = View{width: Fill max_width: 920 height: Fill flow: Down
        brand_bar := View{width: Fill height: 64 flow: Right align: Align{y: 0.5} padding: Inset{left: 20 right: 16} spacing: 8
            AppIcon{width: 26 height: 26 name: "apphub"}
            Text{width: Fill text: "App Hub" draw_text.text_style: theme.font_bold{font_size: 16}}
            source := Pill{width: Fit text: "Live catalog" padding: Inset{left: 10 right: 10} draw_text.text_style.font_size: 13}
            refresh := Plain{width: 44 padding: 0 icon_walk: Walk{width: 17 height: 17} draw_icon +: {svg: crate_resource("self:resources/icons/refresh.svg") color: accent}}
        }
        back_bar := View{visible: false width: Fill height: 44 padding: Inset{left: 10 right: 20} flow: Right
            back := Plain{text: "‹ Back"}
            View{width: Fill height: Fit}
        }
        search_bar := View{visible: false width: Fill height: Fit flow: Down spacing: 14 padding: Inset{left: 20 right: 20 top: 6 bottom: 10}
            Heading{text: "Search" draw_text.text_style: Heavy{font_size: 34}}
            search := TextInputFlat{width: Fill height: 44 empty_text: "Search apps" margin: 0 padding: Inset{left: 14 right: 14 top: 12 bottom: 12}
                draw_bg +: {border_radius: 12.0 color: field color_hover: field color_focus: field color_empty: field border_size: 0.0}
                draw_text +: {color: ink color_hover: ink color_focus: ink color_empty: secondary color_empty_hover: secondary color_empty_focus: secondary text_style: theme.font_regular{font_size: 15}}}
        }
        list := PortalList{width: Fill height: Fill
            Title := Item{padding: Inset{left: 20 right: 20 top: 12 bottom: 24}
                title := Heading{draw_text.text_style: Heavy{font_size: 34}}
                caption := Meta{margin: Inset{top: 5}}
            }
            Section := Item{padding: Inset{left: 20 right: 20 top: 22 bottom: 12}
                title := Heading{}
            }
            Row := AppRow{}
            Hero := Item{padding: Inset{left: 20 right: 20 bottom: 8}
                row := Card{padding: 0 spacing: 0 cursor: MouseCursor.Hand
                    art := Image{width: Fill height: 220 fit: ImageFit.CropToFill src: crate_resource("self:resources/coast.jpg")}
                    View{width: Fill height: Fit flow: Down padding: 22 spacing: 10
                        eyebrow := Meta{text: "EXPLORE OCTOSENSE" draw_text +: {color: #(skin(vm, 0x146958ff, 0x78d7bfff)) text_style: theme.font_bold{font_size: 12}}}
                        title := Heading{text: "Find your next favorite app." draw_text.text_style: Heavy{font_size: 25}}
                        caption := Meta{text: "Find your next favorite place with Maps."}
                        action := Plain{text: "Explore Maps  ›" width: Fit padding: 0}
                    }
                }
            }
            Categories := Item{padding: Inset{left: 16 right: 16 bottom: 8} flow: Right spacing: 2
                all := Plain{text: "All" width: Fill}
                travel := Plain{text: "Explore" width: Fill}
                photo := Plain{text: "Create" width: Fill}
                work := Plain{text: "Work" width: Fill}
            }
            Empty := Item{padding: Inset{left: 24 right: 24 top: 48 bottom: 30}
                Card{padding: 24 spacing: 20 align: Align{x: 0.5}
                    Icon{icon_walk: Walk{width: 68 height: 68} draw_icon +: {svg: crate_resource("self:resources/icons/apps.svg") color: #00b9a2}}
                    title := Heading{draw_text.text_style: Heavy{font_size: 25}}
                    caption := Text{draw_text.color: secondary}
                    retry := Primary{text: "Refresh"}
                    preview := Plain{text: "Explore preview catalog" width: Fill}
                }
            }
            Notice := Item{padding: Inset{left: 20 right: 20 top: 8 bottom: 8}
                caption := Meta{max_lines: 5}
            }
            Detail := Item{
                Card{spacing: 14
                    View{width: Fill height: Fit flow: Right spacing: 16 align: Align{y: 0.5}
                        icon := AppMark{width: 78 height: 78 draw_bg.border_radius: vec4(18.0)}
                        View{width: Fill height: Fit flow: Down spacing: 6
                            name := Heading{}
                            subtitle := Meta{}
                        }
                    }
                    unavailable := Text{visible: false}
                    action := Primary{text: "Get"}
                    // Beside Update: the installed release still opens.
                    open := Plain{width: Fill text: "Open" visible: false}
                    caption := Meta{}
                }
            }
            Copy := Item{padding: Inset{left: 20 right: 20 top: 14 bottom: 8} spacing: 10
                title := Heading{}
                body := Text{}
            }
            Screenshot := Item{padding: Inset{left: 20 right: 20 top: 10 bottom: 10}
                art := Image{width: Fill height: 280 fit: ImageFit.Smallest}
            }
            PreviewArt := Item{padding: Inset{left: 20 right: 20 top: 14 bottom: 8} spacing: 8
                Image{width: Fill height: 210 fit: ImageFit.CropToFill src: crate_resource("self:resources/mountain.jpg")}
                Meta{text: "A glimpse of OctoSense • Preview artwork"}
            }
            Consent := Item{padding: Inset{left: 20 right: 20 top: 12 bottom: 24}
                Card{spacing: 14
                    title := Heading{}
                    permissions := Text{}
                    privacy := Meta{}
                    confirm := Primary{text: "Install"}
                    cancel := Plain{text: "Cancel" width: Fill}
                }
            }
            Space := View{width: Fill height: 20}
        }
        nav := SolidView{width: Fill height: 70 flow: Right padding: Inset{left: 12 right: 12 top: 4 bottom: 8} show_bg: true draw_bg.color: card
            today := Tab{text: "Today" draw_icon.svg: crate_resource("self:resources/icons/today.svg")}
            apps := Tab{text: "Apps" draw_icon.svg: crate_resource("self:resources/icons/apps.svg")}
            search_tab := Tab{text: "Search" draw_icon.svg: crate_resource("self:resources/icons/search.svg")}
            library := Tab{text: "Library" draw_icon.svg: crate_resource("self:resources/icons/library.svg")}
        }
    }

    // A desktop's App Hub is the Mac's App Store: a source list with search at
    // its top, a toolbar, plain rows of apps, a product page. The roles and metrics
    // are those of OctoSense's `apps/interface.splash`, so the store sits among its
    // apps. A phone never reaches this tree (`is_desk`). A shader's border_radius
    // is half the corner it draws: 3 is a 6-point corner.
    let desk = #(is_desk(vm))
    // A Mac's window has no title bar over App Hub (the shell's `unified_chrome`):
    // the sidebar runs to the window's top edge with the lights over it. The `try`
    // matters: a missing property ends a `script_mod!` evaluation.
    let unified = try { theme.ui_unified == true } catch { false }
    let d_ink = theme.color_text
    let d_page = theme.color_bg_app
    let d_light = dot(d_ink, #x36b61200) < 0.5
    let d_lum = dot(d_page, #x36b61200)
    let d_mac = d_lum > 0.86 || d_lum < 0.25
    let d_muted = mix(d_ink, d_page, 0.26)
    let d_dim = mix(d_ink, d_page, 0.56)
    let d_border = mix(d_page, d_ink, 0.14)
    let d_hover = mix(d_page, d_ink, 0.07)
    let d_selected = mix(d_page, d_ink, 0.11)
    let d_field = if d_light { #xffffff } else { mix(d_page, d_ink, 0.07) }
    let d_sidebar = mix(d_page, d_ink, 0.03)
    let d_bar = if d_light { #xededf0 } else { #x303033 }
    let d_button = if d_light { #xffffff } else { mix(d_page, d_ink, 0.16) }
    let d_surface = if d_light { #xffffff } else { mix(d_page, d_ink, 0.07) }
    let d_content = if d_mac { if d_light { #xffffff } else { #x1e1e1e } } else { d_surface }
    let d_link = if d_mac { if d_light { #x0060c0 } else { #x6cb8ff } } else { mix(d_ink, #x356bce, 0.25) }
    let d_primary = #x0071e3
    let d_rim = if d_light { #x00000024 } else { #xffffff2e }
    // A Mac dims the window behind a sheet, as the shell's own sheets do.
    let d_scrim = if d_light { #x00000024 } else { #x00000052 }
    let d_clear = #x00000000

    let DLine = Label{clip_y: false padding: 0 width: Fill text: "" max_lines: 1 text_overflow: Ellipsis
        draw_text +: {color: d_ink text_style: theme.font_regular{font_size: 9.75}}}
    let DNote = DLine{draw_text +: {color: d_muted text_style: theme.font_regular{font_size: 8.25}}}
    let DWrap = Label{clip_y: false padding: 0 width: Fill text: ""
        draw_text +: {color: d_ink text_style: theme.font_regular{font_size: 9.75 line_spacing: 1.35}}}
    let DWrapNote = DWrap{draw_text +: {color: d_muted text_style: theme.font_regular{font_size: 8.25 line_spacing: 1.35}}}
    // A list keeps an item's wrapped line closer than the next item, as a Mac's lists do.
    let DListLine = DWrap{draw_text +: {text_style: theme.font_regular{font_size: 9.75 line_spacing: 1.15}}}
    let DListNote = DWrapNote{draw_text +: {text_style: theme.font_regular{font_size: 8.25 line_spacing: 1.15}}}
    let DHead = Label{clip_y: false padding: 0 width: Fill text: ""
        draw_text +: {color: d_ink text_style: theme.font_bold{font_size: 11.25}}}
    let DRule = SolidView{width: Fill height: 1 draw_bg.color: d_border}
    let DButton = ButtonFlat{height: 28 margin: 0 padding: Inset{left: 12 right: 12} spacing: 6 text: ""
        draw_bg +: {border_radius: 3.0 border_size: 1.0
            border_color: d_border border_color_hover: d_border border_color_down: d_border border_color_focus: d_border border_color_disabled: mix(d_border, d_content, 0.45)
            color: d_button color_hover: mix(d_button, d_ink, 0.04) color_down: mix(d_button, d_ink, 0.1) color_focus: d_button color_disabled: d_clear}
        draw_text +: {color: d_ink color_hover: d_ink color_down: d_ink color_focus: d_ink color_disabled: d_dim
            text_style: theme.font_regular{font_size: 9.75}}
    }
    let DPrimary = DButton{
        draw_bg +: {border_size: 0.0 color: d_primary color_hover: #x006edb color_down: #x0060c0 color_focus: d_primary color_disabled: d_hover}
        draw_text +: {color: #xffffff color_hover: #xffffff color_down: #xffffff color_focus: #xffffff color_disabled: d_dim}
    }
    let DLink = ButtonFlat{height: 22 margin: 0 padding: 0 spacing: 0 text: ""
        draw_bg +: {border_size: 0.0 color: d_clear color_hover: d_clear color_down: d_clear color_focus: d_clear color_disabled: d_clear}
        draw_text +: {color: d_link color_hover: d_link color_down: d_link color_focus: d_link text_style: theme.font_regular{font_size: 9.75}}
    }
    let DTool = ButtonFlat{width: 30 height: 28 margin: 0 padding: 0 spacing: 0 text: "" icon_walk: Walk{width: 20 height: 20}
        draw_bg +: {border_radius: 3.0 border_size: 0.0 color: d_clear color_hover: d_hover color_down: d_selected color_focus: d_clear color_disabled: d_clear}
        draw_icon +: {color: d_ink preserve_viewbox: true}
    }
    // The art under an app's name. A bundle's own icon has its corners cut by a
    // quad in the page's colour with the Mac's tile as its hole, and a hairline
    // rim so that a white or a dark tile keeps its edge: the frame's geometry on the
    // 64-point icon canvas (the shell's `icon_frame`), so these sit level with the
    // shell's art beside them.
    // A point larger than the icon on every side: between two columns a row sits
    // on a part of a pixel, and the picture's edge reaches a pixel further.
    let DMask = View{visible: false width: Fill height: Fill show_bg: true
        margin: Inset{left: -1 top: -1 right: -1 bottom: -1}
        draw_bg +: {
            ground: uniform(d_content)
            rim: uniform(d_rim)
            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let s = (self.rect_size.x - 2.0) / 64.0
                let at = 1.0 + 3.0 * s
                // Past the quad's edge, so that its last pixels are solid too.
                sdf.rect(-2.0, -2.0, self.rect_size.x + 4.0, self.rect_size.y + 4.0)
                sdf.box(at, at, 58.0 * s, 57.0 * s, 6.5 * s)
                sdf.subtract()
                sdf.fill(self.ground)
                sdf.box(at, at, 58.0 * s, 57.0 * s, 6.5 * s)
                return sdf.stroke(self.rim, 0.5)
            }
        }
    }
    let DMark = RoundedAllView{width: 44 height: 44 flow: Overlay align: Center clip_x: false clip_y: false
        draw_bg +: {color: d_clear border_radius: vec4(0.0)}
        mark := View{visible: false width: Fit height: Fit
            Icon{icon_walk: Walk{width: 24 height: 24} draw_icon +: {svg: crate_resource("self:resources/icons/mac-apps.svg") color: d_muted preserve_viewbox: true}}
        }
        builtin := View{visible: false width: Fill height: Fill
            art := AppIcon{width: Fill height: Fill}
        }
        image := Image{visible: false width: Fill height: Fill fit: ImageFit.Smallest}
        mask := DMask{}
    }
    // One app in a list: art, a name, a line, a plain button. The row is plain;
    // a hairline starts at the name and stops at the column's edge.
    let DApp = View{width: Fill height: Fit flow: Down
        row := View{width: Fill height: 62 flow: Right spacing: 12 align: Align{y: 0.5} cursor: MouseCursor.Default
            icon := DMark{}
            View{width: Fill height: Fit flow: Down spacing: 3
                name := DLine{}
                subtitle := DNote{}
            }
            action := DButton{text: "Get"}
        }
        rule := DRule{margin: Inset{left: 56}}
    }
    let DSideTap = ButtonFlat{width: Fill height: 28 margin: 0 padding: Inset{left: 6 right: 8} spacing: 5 align: Align{x: 0.0 y: 0.5} text: ""
        icon_walk: Walk{width: 20 height: 20} label_walk: Walk{width: Fill height: Fit}
        draw_bg +: {border_radius: 3.0 border_size: 0.0 color: d_clear color_hover: d_clear color_down: if d_light { #x00000014 } else { #xffffff1a } color_focus: d_clear}
        draw_text +: {color: d_ink color_hover: d_ink color_down: d_ink color_focus: d_ink text_style: theme.font_regular{font_size: 9.75}}
        draw_icon +: {color: d_link preserve_viewbox: true}
    }
    let DSideMark = RoundedView{visible: false width: Fill height: Fill draw_bg +: {color: d_selected border_radius: 3.0}}
    let DSideSlot = View{width: Fill height: 28 flow: Overlay align: Align{x: 1.0 y: 0.5}}
    let DSideHead = Label{clip_y: false padding: Inset{left: 8 top: 12 bottom: 4} text: ""
        draw_text +: {color: d_muted text_style: theme.font_bold{font_size: 8.25}}}
    let DEmptyGlyph = Icon{width: 44 height: 44 icon_walk: Walk{width: 44 height: 44} draw_icon +: {color: d_muted preserve_viewbox: true}}
    let DEmptyMark = View{visible: false width: 44 height: 44}
    // A product's picture: rounded, a hairline so that a white screenshot keeps its
    // edge on a white page, a tile until it arrives. `AppHubView` sizes it to its row.
    let DShot = Image{width: 232 height: 232 fit: ImageFit.Stretch
        draw_bg +: {
            tile: uniform(d_hover)
            rim: uniform(d_rim)
            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let c = self.get_color()
                sdf.box(0.5, 0.5, self.rect_size.x - 1.0, self.rect_size.y - 1.0, 3.0)
                sdf.fill_keep(mix(self.tile, vec4(c.xyz, 1.0), c.w))
                return sdf.stroke(self.rim, 0.5)
            }
        }
    }
    // A line of a list. The bullet stands alone, so that a long line wraps beside it.
    let DPoint = View{visible: false width: Fill height: Fit flow: Right spacing: 6
        DWrap{width: Fit text: "•"}
        line := DListLine{}
    }
    let DPointNote = View{visible: false width: Fill height: Fit flow: Right spacing: 5
        DWrapNote{width: Fit text: "•"}
        line := DListNote{}
    }
    let DPoints = View{width: Fill height: Fit flow: Down spacing: 5
        b0 := DPoint{} b1 := DPoint{} b2 := DPoint{} b3 := DPoint{} b4 := DPoint{} b5 := DPoint{} b6 := DPoint{} b7 := DPoint{} b8 := DPoint{} b9 := DPoint{} b10 := DPoint{} b11 := DPoint{} b12 := DPoint{} b13 := DPoint{} b14 := DPoint{} b15 := DPoint{}
    }
    let DPointNotes = View{width: Fill height: Fit flow: Down spacing: 5
        b0 := DPointNote{} b1 := DPointNote{} b2 := DPointNote{} b3 := DPointNote{} b4 := DPointNote{} b5 := DPointNote{} b6 := DPointNote{} b7 := DPointNote{} b8 := DPointNote{} b9 := DPointNote{} b10 := DPointNote{} b11 := DPointNote{} b12 := DPointNote{} b13 := DPointNote{} b14 := DPointNote{} b15 := DPointNote{}
    }
    // The Mac's spinner: twelve ticks, the brightest stepping round. `AppHubView`
    // asks for frames while it shows.
    let DSpinner = View{visible: false width: 16 height: 16 show_bg: true
        draw_bg +: {
            color: uniform(d_muted)
            pixel: fn() {
                let center = self.rect_size * 0.5
                let p = self.pos * self.rect_size - center
                let outer = min(center.x, center.y)
                let tick = floor(atan2(p.y, p.x) / 6.2831853 * 12.0 + 0.5)
                let angle = tick / 12.0 * 6.2831853
                let dir = vec2(cos(angle), sin(angle))
                let half = max(outer * 0.09, 0.65)
                let along = dot(p, dir)
                let across = dot(p, vec2(-dir.y, dir.x))
                let d = length(vec2(along - clamp(along, outer * 0.5, outer - half), across)) - half
                let step_now = floor(self.draw_pass.time * 12.0)
                let lit = fract((tick - step_now - 1.0) / 12.0 + 0.001)
                let a = clamp(0.5 - d, 0.0, 1.0) * mix(0.22, 1.0, lit * 12.0 / 11.0)
                return vec4(self.color.rgb * a, a)
            }
        }
    }

    let DeskList = PortalList{width: Fill height: Fill
        Section := View{width: Fill height: Fit padding: Inset{left: 20 right: 20 top: 16 bottom: 2}
            title := DHead{}
        }
        Row := View{width: Fill height: Fit flow: Down padding: Inset{left: 20 right: 20}
            DApp{}
        }
        // Two or three of them across. A cell without an app still holds its width.
        Grid := View{width: Fill height: Fit flow: Right spacing: 24 padding: Inset{left: 20 right: 20}
            c0 := DApp{}
            c1 := DApp{}
            c2 := DApp{visible: false}
        }
        // A catalog that is out of date or out of reach: what it means, and what to do.
        Banner := View{width: Fill height: Fit flow: Down
            View{width: Fill height: Fit flow: Right spacing: 10 align: Align{y: 0.5} padding: Inset{left: 20 right: 20 top: 12 bottom: 12}
                Icon{width: 22 height: 22 icon_walk: Walk{width: 22 height: 22} draw_icon +: {svg: crate_resource("self:resources/icons/mac-warning.svg") color: d_muted preserve_viewbox: true}}
                View{width: Fill height: Fit flow: Down spacing: 2
                    title := DLine{draw_text +: {text_style: theme.font_bold{font_size: 9.75}}}
                    caption := DWrapNote{}
                }
                retry := DButton{text: "Try Again"}
            }
            DRule{}
        }
        // The product page: art, name, publisher; its action at the trailing edge.
        Detail := View{width: Fill height: Fit flow: Down spacing: 10 padding: Inset{left: 20 right: 20 top: 20 bottom: 14}
            View{width: Fill height: Fit flow: Right spacing: 16 align: Align{y: 0.5}
                icon := DMark{width: 96 height: 96}
                View{width: Fill height: Fit flow: Down spacing: 3
                    name := DLine{draw_text +: {text_style: theme.font_bold{font_size: 16.5}}}
                    subtitle := DLine{draw_text.color: d_muted}
                    caption := DNote{}
                }
                View{width: Fit height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                    open := DButton{text: "Open" visible: false}
                    action := DPrimary{text: "Get"}
                }
            }
            // Why it can't be had, and what to do: "Requires a newer version
            // of OctoSense", then the way to Software Update.
            why := View{visible: false width: Fill height: Fit flow: Down spacing: 3
                unavailable := DWrapNote{visible: false}
                remedy := DLink{visible: false height: Fit text: "Software Update…"
                    draw_text +: {text_style: theme.font_regular{font_size: 8.25}}}
            }
        }
        Gallery := View{width: Fill height: Fit flow: Right spacing: 12 padding: Inset{left: 20 right: 20 top: 4 bottom: 12}
            s0 := DShot{}
            s1 := DShot{}
            s2 := DShot{}
        }
        About := View{width: Fill height: Fit flow: Down spacing: 6 padding: Inset{left: 20 right: 20}
            DRule{margin: Inset{bottom: 10}}
            DHead{text: "About"}
            body := DWrap{max_width: 680}
            more := DLink{text: "More…"}
        }
        Copy := View{width: Fill height: Fit flow: Down spacing: 6 padding: Inset{left: 20 right: 20 top: 14}
            DRule{margin: Inset{bottom: 10}}
            title := DHead{}
            body := DWrap{max_width: 680}
        }
        Bullet := View{width: Fill height: Fit flow: Right spacing: 6 padding: Inset{left: 20 right: 20 top: 3 bottom: 3}
            DWrap{width: Fit text: "•"}
            line := DListLine{max_width: 660}
        }
        FirstBullet := View{width: Fill height: Fit flow: Right spacing: 6 padding: Inset{left: 20 right: 20 top: 7 bottom: 3}
            DWrap{width: Fit text: "•"}
            line := DListLine{max_width: 660}
        }
        Info := View{width: Fill height: Fit flow: Down padding: Inset{left: 20 right: 20}
            View{width: Fill height: Fit flow: Right spacing: 12 padding: Inset{top: 7 bottom: 7}
                key := DLine{width: 110 draw_text.color: d_muted}
                value := DWrap{}
            }
            rule := DRule{}
        }
        Notice := View{width: Fill height: Fit flow: Down align: Align{x: 0.5} padding: Inset{left: 20 right: 20 top: 16 bottom: 8}
            caption := DNote{width: Fit}
        }
        UpdatesHead := View{width: Fill height: Fit flow: Right align: Align{y: 0.5} padding: Inset{left: 20 right: 20 top: 12 bottom: 4}
            title := DHead{text: "Updates Available"}
            update_all := DButton{text: "Update All"}
        }
        // Drawn by the pane over the list, centred in what is left of the window.
        Empty := View{width: Fill height: 0}
        Space := View{width: Fill height: 20}
        Top := View{width: Fill height: 8}
    }
    let DeskFrame = View{width: Fill height: Fill flow: Overlay
        View{width: Fill height: Fill flow: Right
            SolidView{width: 208 height: Fill flow: Down spacing: 1 padding: Inset{left: 8 right: 8 top: if unified { 52 } else { 8 } bottom: 8} draw_bg.color: d_sidebar
                // The field's own frame, so holding the keyboard rings it.
                View{width: Fill height: 28 flow: Overlay margin: Inset{bottom: 8}
                    search := TextInput{width: Fill height: 28 margin: 0 padding: Inset{left: 30 right: 25 top: 6 bottom: 6} empty_text: "Search"
                        draw_bg +: {border_radius: 3.0 border_size: 1.0
                            color: d_field color_hover: d_field color_focus: d_field color_empty: d_field
                            border_color: d_border border_color_hover: d_border border_color_focus: d_link border_color_empty: d_border}
                        draw_text +: {color: d_ink color_hover: d_ink color_focus: d_ink color_empty: d_muted color_empty_hover: d_muted color_empty_focus: d_muted text_style: theme.font_regular{font_size: 9.75}}
                    }
                    View{width: Fit height: Fill padding: Inset{left: 7} align: Align{y: 0.5}
                        Icon{width: 18 height: 18 icon_walk: Walk{width: 18 height: 18} draw_icon +: {color: d_muted svg: crate_resource("self:resources/icons/mac-search.svg") preserve_viewbox: true}}
                    }
                    View{width: Fill height: Fill padding: Inset{right: 3} align: Align{x: 1.0 y: 0.5}
                        search_clear := ButtonFlat{visible: false width: 22 height: 22 margin: 0 padding: 0 spacing: 0 text: "" icon_walk: Walk{width: 16 height: 16}
                            draw_bg +: {border_size: 0.0 color: d_clear color_hover: d_clear color_down: d_clear color_focus: d_clear}
                            draw_icon +: {color: d_muted svg: crate_resource("self:resources/icons/mac-clear.svg") preserve_viewbox: true}
                        }
                    }
                }
                DSideSlot{sel_today := DSideMark{} today := DSideTap{text: "Discover" draw_icon +: {svg: crate_resource("self:resources/icons/mac-discover.svg")}}}
                DSideSlot{sel_apps := DSideMark{} apps := DSideTap{text: "Apps" draw_icon +: {svg: crate_resource("self:resources/icons/mac-apps.svg")}}}
                DSideSlot{sel_library := DSideMark{} library := DSideTap{text: "Installed" draw_icon +: {svg: crate_resource("self:resources/icons/mac-installed.svg")}}}
                DSideSlot{sel_updates := DSideMark{} updates := DSideTap{text: "Updates" draw_icon +: {svg: crate_resource("self:resources/icons/mac-updates.svg")}}
                    updates_badge := RoundedView{visible: false width: Fit height: Fit margin: Inset{right: 6} padding: Inset{left: 6 right: 6 top: 2 bottom: 2} new_batch: true
                        draw_bg +: {color: d_selected border_radius: 2.5}
                        updates_count := Label{clip_y: false padding: 0 text: "" draw_text +: {color: d_ink text_style: theme.font_bold{font_size: 8.25}}}
                    }
                }
                DSideHead{text: "Categories"}
                DSideSlot{sel_travel := DSideMark{} cat_travel := DSideTap{text: "Travel" draw_icon +: {svg: crate_resource("self:resources/icons/mac-travel.svg")}}}
                DSideSlot{sel_photo := DSideMark{} cat_photo := DSideTap{text: "Photo & Video" draw_icon +: {svg: crate_resource("self:resources/icons/mac-photo.svg")}}}
                DSideSlot{sel_work := DSideMark{} cat_work := DSideTap{text: "Productivity" draw_icon +: {svg: crate_resource("self:resources/icons/mac-work.svg")}}}
                DSideHead{text: "OctoSense"}
                DSideSlot{sel_included := DSideMark{} included := DSideTap{text: "Included Apps" draw_icon +: {svg: crate_resource("self:resources/icons/mac-included.svg")}}}
            }
            SolidView{width: 1 height: Fill draw_bg.color: d_border}
            View{width: Fill height: Fill flow: Down
                SolidView{width: Fill height: if unified { 52 } else { 40 } flow: Right align: Align{y: 0.5} spacing: 4 padding: Inset{left: 12 right: 10} draw_bg.color: d_bar
                    back := DTool{visible: false reset_hover_on_click: true draw_icon +: {svg: crate_resource("self:resources/icons/mac-back.svg")}}
                    page_title := Label{width: Fill clip_y: false padding: 0 text: "" max_lines: 1 text_overflow: Ellipsis
                        draw_text +: {color: d_ink text_style: theme.font_bold{font_size: 11.25}}}
                    busy_mark := DSpinner{margin: Inset{left: 7 right: 7}}
                    refresh := DTool{draw_icon +: {svg: crate_resource("self:resources/icons/mac-refresh.svg")}}
                }
                DRule{}
                pane := SolidView{width: Fill height: Fill flow: Overlay draw_bg.color: d_content
                    list := DeskList{}
                    // Empty, loading and failure: centred in the pane, no card.
                    empty_pane := View{visible: false width: Fill height: Fill flow: Down align: Align{x: 0.5 y: 0.5} spacing: 6 padding: 24
                        empty_busy := DSpinner{width: 32 height: 32 margin: Inset{bottom: 8}}
                        glyph_search := DEmptyMark{DEmptyGlyph{draw_icon +: {svg: crate_resource("self:resources/icons/mac-search.svg")}}}
                        glyph_apps := DEmptyMark{DEmptyGlyph{draw_icon +: {svg: crate_resource("self:resources/icons/mac-apps.svg")}}}
                        glyph_installed := DEmptyMark{DEmptyGlyph{draw_icon +: {svg: crate_resource("self:resources/icons/mac-installed.svg")}}}
                        glyph_updates := DEmptyMark{DEmptyGlyph{draw_icon +: {svg: crate_resource("self:resources/icons/mac-check.svg")}}}
                        glyph_offline := DEmptyMark{DEmptyGlyph{draw_icon +: {svg: crate_resource("self:resources/icons/mac-offline.svg")}}}
                        empty_title := Label{clip_y: false padding: 0 text: "" draw_text +: {color: d_ink text_style: theme.font_bold{font_size: 11.25}}}
                        empty_note := Label{width: Fill max_width: 360 clip_y: false padding: 0 text: "" align: Align{x: 0.5}
                            draw_text +: {color: d_muted text_style: theme.font_regular{font_size: 9.75 line_spacing: 1.3}}}
                        empty_actions := View{width: Fit height: Fit flow: Right spacing: 8 margin: Inset{top: 10}
                            empty_browse := DButton{visible: false text: "Browse Included Apps"}
                            empty_action := DPrimary{visible: false}
                        }
                    }
                }
            }
        }
        // The Mac's sheet: hung from the toolbar, Cancel beside the default button
        // at the trailing edge. Escape and Return answer it (`desk_key`).
        consent_modal := Modal{
            can_dismiss: false
            align: Align{x: 0.5 y: 0.0}
            bg_view +: {draw_bg +: {color: uniform(d_scrim)}}
            content +: {
                clip_x: false clip_y: false
                sheet := RoundedShadowView{width: 440 height: Fit flow: Down padding: 20 spacing: 10 margin: Inset{top: if unified { 52 } else { 40 }}
                    draw_bg +: {color: d_surface border_radius: 5.0 border_size: 1.0 border_color: d_border
                        shadow_color: #x00000040 shadow_radius: 24.0 shadow_offset: vec2(0.0, 8.0)}
                    View{width: Fill height: Fit flow: Right spacing: 12 align: Align{y: 0.5}
                        icon := DMark{width: 48 height: 48 mask +: {draw_bg +: {ground: uniform(d_surface)}}}
                        View{width: Fill height: Fit flow: Down spacing: 2
                            sheet_title := DLine{draw_text +: {text_style: theme.font_bold{font_size: 9.75}}}
                            sheet_publisher := DNote{}
                        }
                    }
                    // A long list scrolls; the buttons stay in the sheet (`fit_sheet`).
                    sheet_list := ScrollYView{width: Fill height: Fit max_height: 320 flow: Down spacing: 6
                        sheet_intro := DWrap{}
                        sheet_permissions := DPoints{}
                        // A sheet says one thing and asks; the rest is a click away.
                        sheet_more := DLink{margin: Inset{top: 2} text: "Show Privacy Details"}
                        sheet_privacy := DPointNotes{visible: false margin: Inset{top: 2}}
                    }
                    View{width: Fill height: Fit flow: Right spacing: 8 margin: Inset{top: 6} align: Align{x: 1.0 y: 0.5}
                        sheet_cancel := DButton{text: "Cancel"}
                        sheet_confirm := DPrimary{text: "Install"}
                    }
                }
            }
        }
    }

    mod.widgets.AppHubView = set_type_default() do #(AppHubView::register_widget(vm)) {
        width: Fill height: Fill flow: Overlay align: Align{x: 0.5}
        SolidView{width: Fill height: Fill draw_bg.color: if desk { d_content } else { ground }}
        if desk { DeskFrame{} } else { PhoneFrame{} }
    }
}

#[derive(Clone, Default, Debug)]
pub enum AppHubAction {
    Launch(String),
    OpenInstalled(String),
    #[default]
    None,
}
#[derive(Clone, Copy, Default, PartialEq)]
enum Page {
    #[default]
    Today,
    Apps,
    Search,
    Library,
    // A desktop's sidebar has these too.
    Updates,
    Included,
}
#[derive(Clone)]
enum Row {
    Title(String, String),
    Section(String),
    App(Entry),
    Hero(Entry),
    Categories,
    Empty(String, String),
    Notice(String),
    Detail(Entry),
    Copy(String, String),
    Screenshot(String),
    PreviewArt,
    Consent(Entry),
    Space,
    // A desktop's.
    Top,
    Grid(Vec<Entry>),
    Banner(Banner),
    About(String, bool),
    Info(String, String),
    Gallery(Vec<String>),
    Bullet(String),
    /// The first under a heading, as far from it as a paragraph is.
    FirstBullet(String),
    /// The Updates page's heading; Update All, when one of them can go.
    UpdatesHead(bool),
}
impl Row {
    fn template(&self) -> LiveId {
        match self {
            Self::Title(..) => id!(Title),
            Self::Section(..) => id!(Section),
            Self::App(..) => id!(Row),
            Self::Hero(..) => id!(Hero),
            Self::Categories => id!(Categories),
            Self::Empty(..) => id!(Empty),
            Self::Notice(..) => id!(Notice),
            Self::Detail(..) => id!(Detail),
            Self::Copy(..) => id!(Copy),
            Self::Screenshot(..) => id!(Screenshot),
            Self::PreviewArt => id!(PreviewArt),
            Self::Consent(..) => id!(Consent),
            Self::Space => id!(Space),
            Self::Top => id!(Top),
            Self::Grid(..) => id!(Grid),
            Self::Banner(..) => id!(Banner),
            Self::About(..) => id!(About),
            Self::Info(..) => id!(Info),
            Self::Gallery(..) => id!(Gallery),
            Self::Bullet(..) => id!(Bullet),
            Self::FirstBullet(..) => id!(FirstBullet),
            Self::UpdatesHead(..) => id!(UpdatesHead),
        }
    }
}
enum Command {
    Refresh,
    Install(InstallConsent),
    Open(String),
}
enum Reply {
    Catalog(CatalogSnapshot),
    Installed {
        app_id: String,
        snapshot: CatalogSnapshot,
        error: Option<String>,
    },
    Open(String, Result<(), String>),
}

// Owned by the host process, not an App Hub view: downloads can finish
// after the user closes the view and drops its reply receiver.
static COMPLETED_INSTALLS: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// The shell drains this on UI signals, even when no App Hub view is open.
pub fn take_completed_installs() -> Vec<String> {
    std::mem::take(&mut *COMPLETED_INSTALLS.lock().unwrap_or_else(|p| p.into_inner()))
}

fn publish_reply(replies: &Sender<Reply>, reply: Reply) -> bool {
    if let Reply::Installed { app_id, error: None, .. } = &reply {
        COMPLETED_INSTALLS.lock().unwrap_or_else(|p| p.into_inner()).push(app_id.clone());
    }
    let delivered = replies.send(reply).is_ok();
    SignalToUI::set_ui_signal();
    delivered
}

#[derive(Script, Widget)]
pub struct AppHubView {
    #[deref]
    view: View,
    #[rust]
    started: bool,
    #[rust]
    style_changed: bool,
    #[rust]
    page: Page,
    #[rust]
    preview: bool,
    #[rust]
    query: String,
    #[rust]
    category: Option<String>,
    #[rust]
    selected: Option<Entry>,
    #[rust]
    confirming: bool,
    /// The apps an Update All has still to ask about, while it runs.
    #[rust]
    update_queue: Option<Vec<String>>,
    #[rust]
    busy: bool,
    #[rust]
    installing: bool,
    #[rust]
    notice: String,
    #[rust]
    snapshot: Option<CatalogSnapshot>,
    /// The link to App Hub's own agent, when its host grants one: it calls
    /// the read tools ([`crate::ai`]) over it.
    #[rust]
    agent: Option<makepad_app_module::makepad_ai_services::peer::OctosPeer>,
    #[rust]
    rows: Vec<Row>,
    #[rust]
    tx: Option<Sender<Command>>,
    #[rust]
    rx: Option<Receiver<Reply>>,
    #[rust]
    image_bound: HashMap<WidgetUid, String>,
    #[rust]
    svg_requests: HashMap<LiveId, String>,
    #[rust]
    svg_data: HashMap<String, Option<Arc<[u8]>>>,
    /// Art App Hub has shown, by its address: it stands in while App Hub asks
    /// for it again, and when it can't, as a store keeps its icons offline.
    #[rust]
    svg_held: HashMap<String, Arc<[u8]>>,
    /// The desktop's tree is the one on screen (`is_desk`).
    #[rust]
    desk: bool,
    /// How many apps a list's row holds, from the pane's width (0 until drawn).
    #[rust]
    columns: usize,
    /// The pane's width, which a product's pictures fit themselves to.
    #[rust]
    list_width: f64,
    /// The consent sheet's list's height as last applied.
    #[rust]
    sheet_room: f64,
    /// The install sheet's privacy lines are showing.
    #[rust]
    sheet_details: bool,
    /// Where a cleared search goes back to.
    #[rust]
    search_home: Page,
    /// The product page shows the publisher's whole text.
    #[rust]
    about_full: bool,
    /// What the pane says when a list has nothing in it.
    #[rust]
    empty: Option<EmptyPane>,
    /// While the catalog loads, a spinner's frames.
    #[rust]
    spin: NextFrame,
}
impl ScriptHook for AppHubView {
    fn on_after_apply(&mut self, vm: &mut ScriptVm, apply: &Apply, _: &mut Scope, _: ScriptValue) {
        self.desk = is_desk(vm);
        if matches!(apply, Apply::ScriptReapply) {
            self.style_changed = true;
        }
    }
}
impl AppHubView {
    /// The catalog the view shows (`None` until it loads).
    pub fn snapshot(&self) -> Option<&CatalogSnapshot> {
        self.snapshot.as_ref()
    }

    /// Open the link to App Hub's own agent (a host that grants none
    /// refuses it, and nothing arrives on it). Once: a second call keeps
    /// the open link.
    pub fn open_agent(&mut self, cx: &mut Cx) {
        if self.agent.is_none() {
            self.agent = Some(makepad_app_module::makepad_ai_services::peer::OctosPeer::open(cx));
        }
    }

    fn start(&mut self, cx: &mut Cx) {
        if self.started {
            return;
        }
        self.started = true;
        let root = crate::data_root(cx);
        let (tx, commands) = mpsc::channel();
        let (replies, rx) = mpsc::channel();
        match cx
            .thread_spawner()
            .spawn_worker(ThreadOptions::default(), move || {
                let mut backend = Backend::from_environment(root);
                while let Ok(command) = commands.recv() {
                    let reply = match command {
                        Command::Refresh => Reply::Catalog(backend.refresh()),
                        Command::Install(consent) => {
                            let error = backend.install(&consent).err();
                            Reply::Installed {
                                app_id: consent.app_id,
                                snapshot: backend.snapshot(),
                                error,
                            }
                        }
                        Command::Open(id) => {
                            let result = backend.may_open(&id);
                            Reply::Open(id, result)
                        }
                    };
                    if !publish_reply(&replies, reply) {
                        break;
                    }
                }
            }) {
            Ok(handle) => {
                handle.detach();
                self.tx = Some(tx);
                self.rx = Some(rx);
                self.refresh(cx);
            }
            Err(e) => self.notice = format!("Could not start App Hub: {e:?}"),
        }
        self.render(cx);
    }
    pub fn shutdown(&mut self, cx: &mut Cx) {
        for (request, _) in self.svg_requests.drain() {
            cx.cancel_http_request(request);
        }
        self.tx = None;
        self.rx = None;
    }
    fn refresh(&mut self, cx: &mut Cx) {
        if self.busy {
            return;
        }
        self.image_bound.clear();
        for (request, _) in self.svg_requests.drain() {
            cx.cancel_http_request(request);
        }
        // Asked for again, and held until then.
        for (url, data) in self.svg_data.drain() {
            if let Some(data) = data {
                self.svg_held.insert(url, data);
            }
        }
        self.send(Command::Refresh);
    }
    fn send(&mut self, command: Command) {
        if self.tx.as_ref().is_some_and(|tx| tx.send(command).is_ok()) {
            self.busy = true;
        } else {
            self.notice = "App Hub is unavailable. Close it and try again.".into();
        }
    }
    fn drain(&mut self, cx: &mut Cx, _scope: &mut Scope) {
        let replies: Vec<_> = self
            .rx
            .as_ref()
            .map(|rx| rx.try_iter().collect())
            .unwrap_or_default();
        for reply in replies {
            self.busy = false;
            let mut go_on = false;
            match reply {
                Reply::Catalog(snapshot) => {
                    self.notice.clear();
                    self.snapshot = Some(snapshot);
                }
                Reply::Installed { snapshot, error, .. } => {
                    self.installing = false;
                    self.confirming = false;
                    self.snapshot = Some(snapshot);
                    if let Some(error) = error {
                        self.notice = format!("Installation didn’t finish. {error}");
                        // An Update All stops at an update that didn't finish.
                        self.update_queue = None;
                    } else {
                        self.notice = "Installed. Your app is ready to open.".into();
                        go_on = self.update_queue.is_some();
                    }
                }
                Reply::Open(id, result) => match result {
                    Ok(()) => cx.widget_action(self.widget_uid(), AppHubAction::OpenInstalled(id)),
                    Err(error) => self.notice = format!("Could not open this app. {error}"),
                },
            }
            if let Some(entry) = self
                .selected
                .as_mut()
                .filter(|entry| entry.kind == CatalogKind::Live)
            {
                if let Some(current) = self.snapshot.as_ref().and_then(|s| {
                    s.entries
                        .iter()
                        .chain(&s.library)
                        .find(|e| e.id == entry.id)
                }) {
                    *entry = current.clone();
                } else {
                    self.selected = None;
                    self.confirming = false;
                }
            }
            if go_on {
                self.next_update(cx);
            }
            self.render(cx);
        }
    }
    fn entries(&self) -> Vec<Entry> {
        if self.preview {
            if self.desk { included_entries() } else { catalog::preview_entries() }
        } else {
            self.snapshot
                .as_ref()
                .map(|s| {
                    if self.page == Page::Library {
                        s.library.clone()
                    } else {
                        s.entries.clone()
                    }
                })
                .unwrap_or_default()
        }
    }
    fn render(&mut self, cx: &mut Cx) {
        self.style_changed = false;
        if self.desk {
            self.render_desk(cx);
            return;
        }
        self.view.button(cx, ids!(source)).set_text(
            cx,
            if self.preview {
                "Preview catalog"
            } else {
                "Live catalog"
            },
        );
        self.view
            .widget(cx, ids!(back_bar))
            .set_visible(cx, self.selected.is_some());
        self.view
            .widget(cx, ids!(search_bar))
            .set_visible(cx, self.page == Page::Search && self.selected.is_none());
        self.view
            .widget(cx, ids!(nav))
            .set_visible(cx, !self.confirming);
        self.view
            .button(cx, ids!(source))
            .set_enabled(cx, !self.installing);
        self.view
            .button(cx, ids!(refresh))
            .set_enabled(cx, !self.busy);
        for (id, page) in [
            (ids!(today), Page::Today),
            (ids!(apps), Page::Apps),
            (ids!(search_tab), Page::Search),
            (ids!(library), Page::Library),
        ] {
            let (color, face) = cx.with_vm(|vm| {
                let color = if self.page == page { skin(vm, 0x285cd6ff, 0x96b7ffff) }
                    else { skin(vm, 0x53647bff, 0xadb9cdff) };
                let face = if self.page == page { skin(vm, 0xe9eefaff, 0x293c59ff) }
                    else { Vec4f::default() };
                (color, face)
            });
            let mut tab = self.view.widget(cx, id);
            script_apply_eval!(cx,tab,{draw_icon.color: #(color) draw_text +: {color: #(color) color_hover: #(color) color_down: #(color) color_focus: #(color)} draw_bg +: {color: #(face) color_hover: #(face) color_focus: #(face) border_radius: 12}});
        }
        self.rows.clear();
        if !self.notice.is_empty() {
            self.rows.push(Row::Notice(self.notice.clone()));
        }
        if let Some(entry) = self.selected.clone() {
            if entry.kind == CatalogKind::Live {
                if let Some(warning) = self.snapshot.as_ref().and_then(|s| s.warning.clone()) {
                    self.rows.push(Row::Notice(warning));
                }
            }
            self.rows.push(Row::Detail(entry.clone()));
            if self.confirming {
                self.rows.push(Row::Consent(entry));
            } else {
                if entry.kind == CatalogKind::Preview {
                    self.rows.push(Row::PreviewArt);
                }
                for url in &entry.screenshots {
                    self.rows.push(Row::Screenshot(url.clone()));
                }
                self.rows
                    .push(Row::Copy("About".into(), entry.description.clone()));
                self.rows.push(Row::Copy(
                    "Information".into(),
                    format!(
                        "{}\n{}\nVersion {}",
                        entry.publisher, entry.category, entry.version
                    ),
                ));
                if !entry.release_notes.is_empty() {
                    self.rows
                        .push(Row::Copy("What’s new".into(), entry.release_notes.clone()));
                }
                if entry.kind == CatalogKind::Live {
                    self.rows.push(Row::Copy(
                        "Permissions".into(),
                        lines_or(&entry.permissions, "No additional permissions."),
                    ));
                    self.rows.push(Row::Copy(
                        "Privacy".into(),
                        lines_or(&entry.privacy, "No additional privacy information."),
                    ));
                }
            }
        } else {
            let title = match self.page {
                Page::Today => "Today",
                Page::Apps => "Apps",
                Page::Search => "Search",
                // A phone's tab bar has no others.
                Page::Library | Page::Updates | Page::Included => "Library",
            };
            let caption = if self.preview {
                "Preview • Apps included with OctoSense"
            } else if self.page == Page::Library {
                "Your installed apps"
            } else {
                "Discover apps for your OctoSense"
            };
            if self.page != Page::Search {
                self.rows.push(Row::Title(title.into(), caption.into()));
            }
            if !self.preview {
                if let Some(warning) = self.snapshot.as_ref().and_then(|s| s.warning.as_ref()) {
                    self.rows.push(Row::Notice(warning.clone()));
                }
            }
            if matches!(self.page, Page::Apps | Page::Search) {
                self.rows.push(Row::Categories);
            }
            let all = self.entries();
            let entries = catalog::filter_entries(
                &all,
                if self.page == Page::Search {
                    &self.query
                } else {
                    ""
                },
                self.category.as_deref(),
            );
            if entries.is_empty() {
                let (title, caption) = if self.busy && self.snapshot.is_none() {
                    ("Connecting to App Hub", "Checking the latest catalog…")
                } else if (self.page == Page::Search && !self.query.is_empty())
                    || self.category.is_some()
                {
                    ("No matching apps", "Try another search or choose All.")
                } else if self.page == Page::Library {
                    (
                        "Make yourself at home",
                        "Apps you install from the Hub will appear here.",
                    )
                } else if self.snapshot.as_ref().is_some_and(|s| !s.verified) {
                    (
                        "Couldn’t reach App Hub",
                        "Check your connection and try again.",
                    )
                } else {
                    (
                        "Good things are on the way",
                        "Apps published to the OctoSense Hub will appear here.",
                    )
                };
                self.rows.push(Row::Empty(title.into(), caption.into()));
            } else {
                if self.page == Page::Today && self.preview {
                    if let Some(entry) = all.iter().find(|e| e.id == "maps") {
                        self.rows.push(Row::Hero(entry.clone()));
                    }
                }
                if self.page == Page::Today {
                    self.rows.push(Row::Section(
                        if self.preview {
                            "Made for OctoSense"
                        } else {
                            "Explore the Hub"
                        }
                        .into(),
                    ));
                }
                for entry in entries {
                    self.rows.push(Row::App(entry));
                }
            }
            if !self.preview {
                self.rows.push(Row::Notice(
                    if let Some(p) = self.snapshot.as_ref().and_then(|s| s.published.as_ref()) {
                        // Its date, as a phone has always shown it, now that a catalog may carry a time.
                        format!("OctoSense App Hub • Catalog published {}", p.get(..10).unwrap_or(p))
                    } else {
                        "OctoSense App Hub".into()
                    },
                ));
            }
        }
        self.rows.push(Row::Space);
        self.view.redraw(cx);
    }
    fn navigate(&mut self, cx: &mut Cx, page: Page) {
        self.page = page;
        self.selected = None;
        self.confirming = false;
        self.category = None;
        self.reset_scroll(cx);
        self.render(cx);
    }
    fn reset_scroll(&mut self, cx: &mut Cx) {
        self.view
            .portal_list(cx, ids!(list))
            .set_first_id_and_scroll(0, 0.0);
    }
    fn activate(&mut self, cx: &mut Cx, _scope: &mut Scope, entry: Entry) {
        if self.busy && entry.status != EntryStatus::BuiltIn {
            return;
        }
        match entry.status {
            EntryStatus::BuiltIn => {
                cx.widget_action(self.widget_uid(), AppHubAction::Launch(entry.id))
            }
            EntryStatus::Installed => self.send(Command::Open(entry.id)),
            EntryStatus::Available | EntryStatus::UpdateAvailable => {
                if entry.consent.is_some() {
                    self.selected = Some(entry);
                    self.confirming = true;
                    self.reset_scroll(cx);
                } else {
                    self.notice = "Refresh the catalog before installing this app.".into();
                }
            }
            EntryStatus::Unavailable(reason) => self.notice = reason,
        }
        self.render(cx);
    }
    fn bind_image(&mut self, cx: &mut Cx, image: ImageRef, url: Option<&str>) {
        let uid = image.widget_uid();
        let Some(url) = url else {
            image.set_visible(cx, false);
            self.image_bound.remove(&uid);
            return;
        };
        if self.image_bound.get(&uid).is_none_or(|held| held != url) {
            self.image_bound.remove(&uid);
            image.set_texture(cx, None);
            // A built-in's own art, packed into this build: the Dock's picture.
            if let Some(id) = url.strip_prefix(catalog::SYSTEM_ICON) {
                let result = match crate::system_icon(id) {
                    Some(crate::icons::IconData::Svg(svg)) => image.load_svg_from_shared_data(cx, Arc::from(svg.into_bytes())),
                    Some(crate::icons::IconData::Png(png)) => image.load_png_from_data(cx, &png),
                    None => {
                        image.set_visible(cx, false);
                        return;
                    }
                };
                if result.is_err() {
                    image.set_visible(cx, false);
                    return;
                }
                self.image_bound.insert(uid, url.into());
                image.set_visible(cx, true);
                return;
            }
            let svg = url
                .split(['?', '#'])
                .next()
                .unwrap_or(url)
                .to_ascii_lowercase()
                .ends_with(".svg");
            let remote = url.starts_with("https://") || url.starts_with("http://");
            // The framework's async texture cache decodes rasters only. SVGs
            // use Image's vector parser, after an asynchronous HTTP request.
            let result = if svg {
                if !self.svg_data.contains_key(url) {
                    if remote {
                        let request = LiveId::unique();
                        self.svg_data.insert(url.into(), None);
                        self.svg_requests.insert(request, url.into());
                        let mut download = HttpRequest::new(url.into(), HttpMethod::GET);
                        download.set_max_response_body_bytes(1024 * 1024);
                        cx.http_request(request, download);
                    } else {
                        use std::io::Read;
                        let bytes = std::fs::File::open(url).ok().and_then(|file| {
                            let mut bytes = Vec::new();
                            file.take(1024 * 1024 + 1).read_to_end(&mut bytes).ok()?;
                            (bytes.len() <= 1024 * 1024).then(|| Arc::from(bytes))
                        });
                        if bytes.is_some() {
                            self.svg_held.remove(url);
                        }
                        self.svg_data.insert(url.into(), bytes);
                    }
                }
                let data = match self.svg_data.get(url) {
                    Some(Some(data)) => Some(data.clone()),
                    // Still on its way, or it couldn't come: what was shown before.
                    _ => self.svg_held.get(url).cloned(),
                };
                let Some(data) = data else {
                    image.set_visible(cx, false);
                    return;
                };
                image.load_svg_from_shared_data(cx, data)
            } else if remote {
                image.load_image_http_by_url_async(cx, url)
            } else {
                image.load_image_file_by_path_async(cx, std::path::Path::new(url))
            };
            if result.is_err() {
                image.set_visible(cx, false);
                return;
            }
            self.image_bound.insert(uid, url.into());
        }
        image.set_visible(cx, true);
    }
    fn receive_svg(&mut self, cx: &mut Cx, responses: &NetworkResponsesEvent) {
        for response in responses {
            let (request, body) = match response {
                NetworkResponse::HttpResponse {
                    request_id,
                    response,
                }
                | NetworkResponse::HttpStreamComplete {
                    request_id,
                    response,
                } => (
                    request_id,
                    response
                        .body
                        .as_ref()
                        .filter(|body| {
                            (200..300).contains(&response.status_code) && body.len() <= 1024 * 1024
                        })
                        .cloned(),
                ),
                NetworkResponse::HttpError { request_id, .. } => (request_id, None),
                _ => continue,
            };
            if let Some(url) = self.svg_requests.remove(request) {
                if body.is_some() {
                    // What stood in for it gives way to what came.
                    self.svg_held.remove(&url);
                    self.image_bound.retain(|_, held| *held != url);
                }
                self.svg_data.insert(url, body);
                self.view.redraw(cx);
            }
        }
    }
    fn fill_entry(&mut self, cx: &mut Cx, item: &WidgetRef, entry: &Entry) {
        item.label(cx, ids!(name)).set_text(cx, &entry.name);
        if self.desk {
            let line = if self.page == Page::Updates && self.selected.is_none() { update_line(entry) } else { plain_line(entry) };
            item.label(cx, ids!(subtitle)).set_text(cx, &line);
        } else {
            item.label(cx, ids!(subtitle)).set_text(cx, &entry.subtitle);
        }
        // Details must explain a disabled action without requiring a click.
        // Clear the text too: PortalList can reuse this widget for another app.
        let reason = match &entry.status {
            EntryStatus::Unavailable(reason) => Some(reason.as_str()),
            _ => None,
        };
        item.label(cx, ids!(unavailable)).set_text(cx, reason.unwrap_or_default());
        item.widget(cx, ids!(unavailable)).set_visible(cx, reason.is_some());
        let enabled = (!self.busy || entry.status == EntryStatus::BuiltIn) && can_activate(entry);
        item.button(cx, ids!(action)).set_enabled(cx, enabled);
        if self.desk {
            item.widget(cx, ids!(action)).set_disabled(cx, !enabled);
        }
        item.button(cx, ids!(action)).set_text(
            cx,
            if self.installing && self.selected.as_ref().is_some_and(|e| e.id == entry.id) {
                "Installing…"
            } else if self.desk {
                desk_button_text(entry)
            } else {
                button_text(entry)
            },
        );
        // A built-in draws the shell's launcher art, or its bundle's own icon
        // when the shell has none for it.
        let known = entry.kind == CatalogKind::Preview && entry.icon.is_none();
        let color = if self.desk {
            // Under a bundle's art, the backing the shell's icons have; a neutral
            // tile until there is art. The mask cuts either to the Mac's shape.
            let image = item.image(cx, ids!(icon.image));
            let shown = self.image_bound.get(&image.widget_uid()).map(String::as_str) == entry.icon.as_deref()
                && entry.icon.is_some() && image.has_content();
            if known {
                Vec4f::from_u32(0)
            } else if shown {
                Vec4f::from_u32(0xfffdf7ff)
            } else {
                cx.with_vm(|vm| skin(vm, 0xe8e8ebff, 0x3a3a3cff))
            }
        } else {
            Vec4f::from_u32(if known || entry.icon.is_some() { 0 } else { match entry.category.as_str() {
                "travel" => 0x00ac96ff,
                "news" => 0xf44863ff,
                "photo-video" => 0x8c68dfff,
                "productivity" => 0x2587e8ff,
                _ => 0x00a78fff,
            }})
        };
        let mut icon = item.widget(cx, ids!(icon));
        script_apply_eval!(cx,icon,{draw_bg.color: #(color)});
        self.bind_image(cx, item.image(cx, ids!(icon.image)), entry.icon.as_deref());
        let artwork = item.image(cx, ids!(icon.image));
        let loaded = self
            .image_bound
            .get(&artwork.widget_uid())
            .map(String::as_str)
            == entry.icon.as_deref()
            && entry.icon.is_some()
            && artwork.has_content();
        item.widget(cx, ids!(icon.mark))
            .set_visible(cx, !known && !loaded);
        item.widget(cx, ids!(icon.builtin)).set_visible(cx, known);
        item.widget(cx, ids!(icon.mask)).set_visible(cx, !known);
        if known {
            if let Some(mut art) = item.widget(cx, ids!(icon.builtin.art)).borrow_mut::<app_icon::AppIcon>() {
                art.set_name(cx, &entry.id);
            }
        }
    }
    fn draw_rows(&mut self, cx: &mut Cx2d, list: &mut PortalList) {
        if self.desk {
            self.fit_columns(cx);
        }
        list.set_item_range(cx, 0, self.rows.len());
        while let Some(index) = list.next_visible_item(cx) {
            let Some(row) = self.rows.get(index).cloned() else {
                continue;
            };
            let item = list.item(cx, index, row.template());
            match row {
                Row::Title(title, caption) | Row::Empty(title, caption) => {
                    item.label(cx, ids!(title)).set_text(cx, &title);
                    item.label(cx, ids!(caption)).set_text(cx, &caption);
                    item.widget(cx, ids!(preview))
                        .set_visible(cx, !self.preview);
                    item.button(cx, ids!(retry)).set_text(
                        cx,
                        if (self.page == Page::Search && !self.query.is_empty())
                            || self.category.is_some()
                        {
                            "Clear filters"
                        } else if self.busy {
                            "Connecting…"
                        } else {
                            "Refresh"
                        },
                    );
                }
                Row::Section(title) => item.label(cx, ids!(title)).set_text(cx, &title),
                Row::UpdatesHead(ready) => {
                    item.button(cx, ids!(update_all)).set_enabled(cx, ready);
                    item.widget(cx, ids!(update_all)).set_disabled(cx, !ready);
                }
                Row::App(entry) | Row::Detail(entry) => {
                    self.fill_entry(cx, &item, &entry);
                    // Only the detail card has it; a list row's button is enough.
                    item.widget(cx, ids!(open)).set_visible(cx, shows_open(&entry));
                    if self.desk {
                        self.finish_desk_entry(cx, &item, &entry, index);
                    } else {
                        item.label(cx, ids!(caption)).set_text(
                            cx,
                            if entry.kind == CatalogKind::Preview {
                                "Included with OctoSense • Preview"
                            } else {
                                &entry.publisher
                            },
                        );
                    }
                }
                Row::Grid(entries) => self.draw_grid(cx, &item, &entries, index),
                Row::Banner(banner) => {
                    item.label(cx, ids!(title)).set_text(cx, &banner.title);
                    item.label(cx, ids!(caption)).set_text(cx, &banner.note);
                    item.widget(cx, ids!(retry)).set_visible(cx, banner.retry != Retry::None);
                }
                Row::About(body, more) => {
                    item.label(cx, ids!(body)).set_text(cx, &body);
                    item.widget(cx, ids!(more)).set_visible(cx, more);
                }
                Row::Info(key, value) => {
                    item.label(cx, ids!(key)).set_text(cx, &key);
                    item.label(cx, ids!(value)).set_text(cx, &value);
                    let last = !matches!(self.rows.get(index + 1), Some(Row::Info(..)));
                    item.widget(cx, ids!(rule)).set_visible(cx, !last);
                }
                Row::Gallery(urls) => {
                    let mut tiles = Vec::new();
                    for (i, id) in [ids!(s0), ids!(s1), ids!(s2)].into_iter().enumerate() {
                        let image = item.image(cx, id);
                        self.bind_image(cx, image.clone(), urls.get(i).map(String::as_str));
                        if i < urls.len() {
                            tiles.push(image);
                        }
                    }
                    self.size_gallery(cx, &tiles);
                }
                Row::Bullet(line) | Row::FirstBullet(line) => item.label(cx, ids!(line)).set_text(cx, &line),
                Row::Notice(text) => item.label(cx, ids!(caption)).set_text(cx, &text),
                Row::Copy(title, body) => {
                    item.label(cx, ids!(title)).set_text(cx, &title);
                    item.label(cx, ids!(body)).set_text(cx, &body);
                    if self.desk {
                        item.widget(cx, ids!(body)).set_visible(cx, !body.is_empty());
                    }
                }
                Row::Screenshot(url) => {
                    let image = item.image(cx, ids!(art));
                    self.bind_image(cx, image, Some(&url));
                }
                Row::Consent(entry) => {
                    item.label(cx, ids!(title))
                        .set_text(cx, &format!("Install {}?", entry.name));
                    item.label(cx, ids!(permissions)).set_text(
                        cx,
                        &format!(
                            "This app will have access to:\n\n{}",
                            lines_or(&entry.permissions, "No additional permissions.")
                        ),
                    );
                    item.label(cx, ids!(privacy))
                        .set_text(cx, &lines_or(&entry.privacy, ""));
                    item.button(cx, ids!(confirm))
                        .set_enabled(cx, !self.busy && entry.consent.is_some());
                    item.button(cx, ids!(cancel))
                        .set_enabled(cx, !self.installing);
                    item.button(cx, ids!(confirm)).set_text(
                        cx,
                        if self.installing {
                            "Installing…"
                        } else {
                            "Install"
                        },
                    );
                }
                Row::Categories => {
                    for (id, category) in [
                        (ids!(all), None),
                        (ids!(travel), Some("travel")),
                        (ids!(photo), Some("photo-video")),
                        (ids!(work), Some("productivity")),
                    ] {
                        let color = Vec4f::from_u32(if self.category.as_deref() == category {
                            0x009b84ff
                        } else {
                            0x687b97ff
                        });
                        let mut button = item.widget(cx, id);
                        script_apply_eval!(cx,button,{draw_text.color: #(color)});
                    }
                }
                _ => {}
            }
            item.draw_all(cx, &mut Scope::empty());
        }
    }
    fn actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        if self.desk {
            self.desk_actions(cx, actions, scope);
            return;
        }
        if self.view.button(cx, ids!(source)).clicked(actions) && !self.installing {
            self.preview = !self.preview;
            self.selected = None;
            self.confirming = false;
            self.category = None;
            self.notice.clear();
            self.reset_scroll(cx);
            self.render(cx);
        }
        for (id, page) in [
            (ids!(today), Page::Today),
            (ids!(apps), Page::Apps),
            (ids!(search_tab), Page::Search),
            (ids!(library), Page::Library),
        ] {
            if self.view.button(cx, id).clicked(actions) {
                self.navigate(cx, page);
            }
        }
        if self.view.button(cx, ids!(back)).clicked(actions) {
            self.back(cx);
        }
        if self.view.button(cx, ids!(refresh)).clicked(actions) {
            self.refresh(cx);
            self.render(cx);
        }
        if let Some(query) = self.view.text_input(cx, ids!(search)).changed(actions) {
            self.query = query;
            self.reset_scroll(cx);
            self.render(cx);
        }
        let items = self
            .view
            .portal_list(cx, ids!(list))
            .items_with_actions(actions);
        for (index, item) in items {
            let Some(row) = self.rows.get(index).cloned() else {
                continue;
            };
            match row {
                Row::App(entry) | Row::Detail(entry) | Row::Hero(entry) => {
                    if item.button(cx, ids!(open)).clicked(actions) {
                        if shows_open(&entry) && !self.busy {
                            self.send(Command::Open(entry.id.clone()));
                        }
                    } else if item.button(cx, ids!(action)).clicked(actions) {
                        self.activate(cx, scope, entry);
                    } else if item
                        .widget(cx, ids!(row))
                        .as_view()
                        .finger_up(actions)
                        .is_some_and(|up| up.was_tap())
                    {
                        self.selected = Some(entry);
                        self.confirming = false;
                        self.reset_scroll(cx);
                        self.render(cx);
                    }
                }
                Row::Empty(..) => {
                    if item.button(cx, ids!(retry)).clicked(actions) {
                        if (self.page == Page::Search && !self.query.is_empty())
                            || self.category.is_some()
                        {
                            self.query.clear();
                            self.category = None;
                            self.view.text_input(cx, ids!(search)).set_text(cx, "");
                        } else {
                            self.refresh(cx);
                        }
                        self.render(cx);
                    }
                    if item.button(cx, ids!(preview)).clicked(actions) {
                        self.preview = true;
                        self.navigate(cx, Page::Today);
                    }
                }
                Row::Categories => {
                    for (id, category) in [
                        (ids!(all), None),
                        (ids!(travel), Some("travel")),
                        (ids!(photo), Some("photo-video")),
                        (ids!(work), Some("productivity")),
                    ] {
                        if item.button(cx, id).clicked(actions) {
                            self.category = category.map(str::to_owned);
                            self.render(cx);
                        }
                    }
                }
                Row::Consent(entry) => {
                    if item.button(cx, ids!(confirm)).clicked(actions) && !self.busy {
                        if let Some(consent) = entry.consent {
                            self.installing = true;
                            self.send(Command::Install(consent));
                            self.render(cx);
                        }
                    }
                    if item.button(cx, ids!(cancel)).clicked(actions) && !self.installing {
                        self.confirming = false;
                        self.render(cx);
                    }
                }
                _ => {}
            }
        }
    }
    fn back(&mut self, cx: &mut Cx) -> bool {
        if self.installing {
            return true;
        }
        if self.confirming {
            self.confirming = false;
            if self.update_queue.take().is_some() {
                self.selected = None;
            }
        } else if self.selected.take().is_none() {
            return false;
        } else {
            self.about_full = false;
            self.notice.clear();
        }
        self.reset_scroll(cx);
        self.render(cx);
        true
    }
}
// ---- The desktop: the Mac's App Store over the same catalog and the same
// worker. A phone never comes here. ----

/// The sidebar's rows.
#[derive(Clone, Copy, PartialEq)]
enum Side {
    Discover,
    Apps,
    Installed,
    Updates,
    Travel,
    Photo,
    Work,
    Included,
}
impl Side {
    fn page(self) -> Page {
        match self {
            Self::Discover => Page::Today,
            Self::Apps | Self::Travel | Self::Photo | Self::Work => Page::Apps,
            Self::Installed => Page::Library,
            Self::Updates => Page::Updates,
            Self::Included => Page::Included,
        }
    }
    fn category(self) -> Option<&'static str> {
        match self {
            Self::Travel => Some("travel"),
            Self::Photo => Some("photo-video"),
            Self::Work => Some("productivity"),
            _ => None,
        }
    }
}
/// What a banner's Try Again does.
#[derive(Clone, Copy, PartialEq)]
enum Retry {
    None,
    Catalog,
    Install,
}
/// A catalog or an install that needs the person: a title, what it means and,
/// when there is one, the action.
#[derive(Clone)]
struct Banner {
    title: String,
    note: String,
    retry: Retry,
}
#[derive(Clone, Copy, PartialEq)]
enum EmptyGlyph {
    Busy,
    Search,
    Apps,
    Installed,
    Updates,
    Offline,
}
#[derive(Clone, Copy, PartialEq)]
enum EmptyAction {
    Refresh,
    Discover,
}
/// A list with nothing in it, in the UiEmpty look.
#[derive(Clone)]
struct EmptyPane {
    glyph: EmptyGlyph,
    title: String,
    note: String,
    action: Option<(&'static str, EmptyAction)>,
    browse: bool,
}

impl AppHubView {
    /// The sidebar's row for what is open: a search has none.
    fn side_on(&self) -> Option<Side> {
        match self.page {
            Page::Today => Some(Side::Discover),
            Page::Apps => Some(match self.category.as_deref() {
                Some("travel") => Side::Travel,
                Some("photo-video") => Side::Photo,
                Some("productivity") => Side::Work,
                _ => Side::Apps,
            }),
            Page::Search => None,
            Page::Library => Some(Side::Installed),
            Page::Updates => Some(Side::Updates),
            Page::Included => Some(Side::Included),
        }
    }
    fn desk_title(&self) -> String {
        // A product page has its back button; its name is on the page.
        if self.selected.is_some() {
            return String::new();
        }
        match self.page {
            Page::Today => "Discover".into(),
            Page::Apps => self.category.as_deref().map_or("Apps".into(), category_name),
            Page::Search if self.query.trim().is_empty() => "Search".into(),
            Page::Search => format!("Results for “{}”", self.query.trim()),
            Page::Library => "Installed".into(),
            Page::Updates => "Updates".into(),
            Page::Included => "Included Apps".into(),
        }
    }
    /// Installed apps the catalog has a newer release of.
    fn updates(&self) -> Vec<Entry> {
        self.snapshot.as_ref().map_or_else(Vec::new, |s| {
            s.library.iter().filter(|e| e.status == EntryStatus::UpdateAvailable).cloned().collect()
        })
    }
    fn render_desk(&mut self, cx: &mut Cx) {
        let title = self.desk_title();
        self.view.label(cx, ids!(page_title)).set_text(cx, &title);
        self.view.widget(cx, ids!(back)).set_visible(cx, self.selected.is_some());
        // Nothing on the included page comes from the catalog.
        self.view.widget(cx, ids!(refresh)).set_visible(cx, !self.busy && self.selected.is_none() && self.page != Page::Included);
        self.view.widget(cx, ids!(search_clear)).set_visible(cx, !self.query.is_empty());
        let on = self.side_on();
        for (id, side) in [
            (ids!(sel_today), Side::Discover),
            (ids!(sel_apps), Side::Apps),
            (ids!(sel_library), Side::Installed),
            (ids!(sel_updates), Side::Updates),
            (ids!(sel_travel), Side::Travel),
            (ids!(sel_photo), Side::Photo),
            (ids!(sel_work), Side::Work),
            (ids!(sel_included), Side::Included),
        ] {
            self.view.widget(cx, id).set_visible(cx, on == Some(side));
        }
        let waiting = self.updates().len();
        self.view.widget(cx, ids!(updates_badge)).set_visible(cx, waiting > 0);
        self.view.label(cx, ids!(updates_count)).set_text(cx, &waiting.to_string());
        self.desk_rows();
        // The pane has its own spinner while there is nothing to show yet.
        let pane_spins = self.empty.as_ref().is_some_and(|pane| pane.glyph == EmptyGlyph::Busy);
        self.view.widget(cx, ids!(busy_mark)).set_visible(cx, self.busy && !pane_spins);
        self.show_empty(cx);
        self.show_sheet(cx);
        self.view.redraw(cx);
    }
    fn desk_rows(&mut self) {
        self.rows.clear();
        self.empty = None;
        // A finished install says so with the page's own button: Open.
        if !self.notice.is_empty() && !self.notice.starts_with("Installed.") {
            self.rows.push(Row::Banner(notice_banner(&self.notice)));
        }
        let warning = self.snapshot.as_ref().and_then(|s| s.warning.clone());
        let published = self.snapshot.as_ref().and_then(|s| s.published.clone());
        if let Some(entry) = self.selected.clone() {
            if let (Some(warning), CatalogKind::Live) = (&warning, entry.kind) {
                self.rows.push(Row::Banner(warning_banner(warning, published.as_deref())));
            }
            self.push_product_page(entry);
            self.rows.push(Row::Space);
            return;
        }
        let live = self.snapshot.as_ref().map_or_else(Vec::new, |s| s.entries.clone());
        let mut sections: Vec<(Option<String>, Vec<Entry>)> = match self.page {
            Page::Today => group_by_category(self.entries()),
            Page::Apps => vec![(None, catalog::filter_entries(&self.entries(), "", self.category.as_deref()))],
            // Everything a person could mean: the catalog and what came with OctoSense.
            Page::Search => vec![
                (Some("App Hub".to_string()), catalog::filter_entries(&live, &self.query, None)),
                (Some("Included with OctoSense".to_string()), catalog::filter_entries(&included_entries(), &self.query, None)),
            ],
            Page::Library => vec![(None, self.entries())],
            Page::Updates => vec![(None, self.updates())],
            Page::Included => vec![(None, included_entries())],
        };
        sections.retain(|(_, entries)| !entries.is_empty());
        let about_catalog = !self.preview && self.page != Page::Included;
        // A catalog in doubt is told where it changes what can be got: the catalog's
        // pages, and an update on offer. An empty page's own pane says the rest.
        let affected = match self.page {
            Page::Today | Page::Apps => !sections.is_empty(),
            Page::Library | Page::Updates => sections
                .iter()
                .any(|(_, entries)| entries.iter().any(|e| e.status == EntryStatus::UpdateAvailable)),
            Page::Search => true,
            Page::Included => false,
        };
        if let Some(warning) = &warning {
            let banner = warning_banner(warning, published.as_deref());
            // Reopening holds up every page.
            if about_catalog && (affected || banner.retry == Retry::None) {
                self.rows.push(Row::Banner(banner));
            }
        }
        if sections.is_empty() {
            let pane = self.desk_empty();
            self.rows.push(Row::Empty(pane.title.clone(), pane.note.clone()));
            self.empty = Some(pane);
            return;
        }
        if self.page == Page::Updates {
            let ready = !self.busy && !self.installing && sections.iter().any(|(_, entries)| entries.iter().any(can_activate));
            self.rows.push(Row::UpdatesHead(ready));
        } else if self.rows.is_empty() && sections[0].0.is_none() {
            // A list with no heading still starts a little below the toolbar.
            self.rows.push(Row::Top);
        }
        for (title, entries) in sections {
            if let Some(title) = title {
                self.rows.push(Row::Section(title));
            }
            // What a person already has is one list, and so are a search's answers and
            // a section of one app; the rest of the catalog is a grid.
            let list = matches!(self.page, Page::Library | Page::Updates) || !self.query.trim().is_empty() || entries.len() == 1;
            let columns = if list { 1 } else { self.columns.max(1) };
            if columns == 1 {
                self.rows.extend(entries.into_iter().map(Row::App));
            } else {
                self.rows.extend(entries.chunks(columns).map(|row| Row::Grid(row.to_vec())));
            }
        }
        // A banner above has already said how old the catalog is.
        if about_catalog && warning.is_none() && self.page != Page::Library && self.page != Page::Updates {
            if let Some(date) = published.as_deref().and_then(date_words) {
                self.rows.push(Row::Notice(format!("Catalog updated {date}")));
            }
        }
        self.rows.push(Row::Space);
    }
    fn desk_empty(&self) -> EmptyPane {
        let pane = |glyph, title: &str, note: &str, action, browse| EmptyPane {
            glyph, title: title.into(), note: note.into(), action, browse,
        };
        let query = self.query.trim();
        if self.page == Page::Search && !query.is_empty() {
            return pane(EmptyGlyph::Search, &format!("No Results for “{query}”"), "Check the spelling or try a new search.", None, false);
        }
        if self.busy && self.snapshot.is_none() {
            return pane(EmptyGlyph::Busy, "Connecting to App Hub…", "Checking for the latest apps.", None, false);
        }
        match self.page {
            Page::Library => pane(EmptyGlyph::Installed, "No Apps Installed", "Apps you get from App Hub appear here.", Some(("Discover Apps", EmptyAction::Discover)), false),
            // With nothing installed there is nothing to be behind.
            Page::Updates if self.snapshot.as_ref().is_some_and(|s| s.library.is_empty()) =>
                pane(EmptyGlyph::Updates, "No Updates Available", "Updates for the apps you get from App Hub appear here.", Some(("Discover Apps", EmptyAction::Discover)), false),
            // A catalog in doubt can't say they are up to date.
            Page::Updates if self.snapshot.as_ref().is_some_and(|s| s.warning.is_some()) =>
                pane(EmptyGlyph::Offline, "Can’t Check for Updates", "There may be newer versions than App Hub can see.", Some(("Try Again", EmptyAction::Refresh)), false),
            // The toolbar's button checks again, and Command-R.
            Page::Updates => pane(EmptyGlyph::Updates, "No Updates Available", "Your apps are up to date.", None, false),
            _ if self.snapshot.as_ref().is_none_or(|s| !s.verified) => pane(EmptyGlyph::Offline, "Can’t Connect to App Hub", "Check your connection and try again.", Some(("Try Again", EmptyAction::Refresh)), true),
            _ if self.category.is_some() => pane(EmptyGlyph::Apps, &format!("No {} Apps", category_name(self.category.as_deref().unwrap_or_default())), "Apps in this category appear here when they are published.", None, false),
            _ => pane(EmptyGlyph::Apps, "No Apps Yet", "Apps published to App Hub appear here.", None, true),
        }
    }
    fn show_empty(&mut self, cx: &mut Cx) {
        let Some(pane) = self.empty.clone() else {
            self.view.widget(cx, ids!(empty_pane)).set_visible(cx, false);
            return;
        };
        self.view.widget(cx, ids!(empty_pane)).set_visible(cx, true);
        for (id, glyph) in [
            (ids!(glyph_search), EmptyGlyph::Search),
            (ids!(glyph_apps), EmptyGlyph::Apps),
            (ids!(glyph_installed), EmptyGlyph::Installed),
            (ids!(glyph_updates), EmptyGlyph::Updates),
            (ids!(glyph_offline), EmptyGlyph::Offline),
            (ids!(empty_busy), EmptyGlyph::Busy),
        ] {
            self.view.widget(cx, id).set_visible(cx, pane.glyph == glyph);
        }
        self.view.label(cx, ids!(empty_title)).set_text(cx, &pane.title);
        self.view.label(cx, ids!(empty_note)).set_text(cx, &pane.note);
        let action = self.view.button(cx, ids!(empty_action));
        action.set_text(cx, pane.action.map_or("", |(text, _)| text));
        self.view.widget(cx, ids!(empty_action)).set_visible(cx, pane.action.is_some());
        self.view.widget(cx, ids!(empty_browse)).set_visible(cx, pane.browse);
        action.set_enabled(cx, !self.busy);
        self.view.widget(cx, ids!(empty_action)).set_disabled(cx, self.busy);
    }
    /// The consent sheet: open exactly while the person is deciding.
    fn show_sheet(&mut self, cx: &mut Cx) {
        let modal = self.view.modal(cx, ids!(consent_modal));
        let entry = self.selected.clone().filter(|_| self.confirming);
        let Some(entry) = entry else {
            modal.close(cx);
            return;
        };
        if !modal.is_open() {
            self.sheet_details = false;
        }
        let update = entry.status == EntryStatus::UpdateAvailable;
        self.view.label(cx, ids!(sheet_title)).set_text(cx, &format!(
            "{} {}?", if update { "Update" } else { "Install" }, entry.name));
        self.view.label(cx, ids!(sheet_publisher)).set_text(cx, &entry.publisher);
        let permissions = desk_permissions(&entry);
        self.view.label(cx, ids!(sheet_intro)).set_text(cx, if permissions.is_empty() {
            "No additional permissions."
        } else {
            "This app will have access to:"
        });
        let list = self.view.widget(cx, ids!(sheet_permissions));
        fill_points(cx, &list, &permissions);
        let privacy = desk_privacy(&entry);
        let list = self.view.widget(cx, ids!(sheet_privacy));
        fill_points(cx, &list, &privacy);
        self.view.widget(cx, ids!(sheet_more)).set_visible(cx, !privacy.is_empty());
        self.view.button(cx, ids!(sheet_more)).set_text(cx, if self.sheet_details { "Hide Privacy Details" } else { "Show Privacy Details" });
        self.view.widget(cx, ids!(sheet_privacy)).set_visible(cx, self.sheet_details);
        let confirm = self.view.button(cx, ids!(sheet_confirm));
        let ready = !self.busy && entry.consent.is_some();
        confirm.set_enabled(cx, ready);
        self.view.widget(cx, ids!(sheet_confirm)).set_disabled(cx, !ready);
        confirm.set_text(cx, match (self.installing, update) {
            (true, true) => "Updating…",
            (true, false) => "Installing…",
            (false, true) => "Update",
            (false, false) => "Install",
        });
        self.view.button(cx, ids!(sheet_cancel)).set_enabled(cx, !self.installing);
        self.view.widget(cx, ids!(sheet_cancel)).set_disabled(cx, self.installing);
        let sheet = self.view.widget(cx, ids!(sheet));
        self.fill_entry(cx, &sheet, &entry);
        if !modal.is_open() {
            modal.open(cx);
        }
    }
    fn push_product_page(&mut self, entry: Entry) {
        self.rows.push(Row::Detail(entry.clone()));
        let per_row = if self.columns == 1 { 2 } else { 3 };
        self.rows.extend(entry.screenshots.chunks(per_row).map(|urls| Row::Gallery(urls.to_vec())));
        let description = entry.description.strip_suffix(catalog::PREVIEW_NOTE).unwrap_or(&entry.description);
        let plain = plain_about(description, &entry);
        let trimmed = plain != description || plain_notes(&entry.release_notes) != entry.release_notes.trim();
        if self.about_full {
            self.rows.push(Row::About(description.to_string(), false));
        } else {
            self.rows.push(Row::About(plain, trimmed));
        }
        let notes = if self.about_full { entry.release_notes.clone() } else { plain_notes(&entry.release_notes) };
        if !notes.is_empty() {
            self.rows.push(Row::Copy("What’s New".into(), notes));
        }
        self.rows.push(Row::Copy("Information".into(), String::new()));
        self.rows.push(Row::Info("Publisher".into(), if entry.publisher.is_empty() { "Unknown".into() } else { entry.publisher.clone() }));
        self.rows.push(Row::Info("Category".into(), category_name(&entry.category)));
        // What came with OctoSense has its version; the page's header says it did.
        if entry.kind != CatalogKind::Preview {
            self.rows.push(Row::Info("Version".into(), entry.version.clone()));
        }
        if entry.kind == CatalogKind::Live {
            self.push_policy("Permissions", desk_permissions(&entry), "No additional permissions.");
            self.push_policy("Privacy", desk_privacy(&entry), "No additional privacy information.");
        }
    }
    fn push_policy(&mut self, title: &str, lines: Vec<String>, empty: &str) {
        self.rows.push(Row::Copy(title.into(), if lines.is_empty() { empty.into() } else { String::new() }));
        self.rows.extend(lines.into_iter().enumerate().map(|(index, line)| {
            if index == 0 { Row::FirstBullet(line) } else { Row::Bullet(line) }
        }));
    }
    /// A row of a product's pictures, side by side at one height. A picture still
    /// on its way keeps a tile's place, as wide as most of them turn out to be.
    fn size_gallery(&self, cx: &mut Cx, tiles: &[ImageRef]) {
        let aspects: Vec<f64> = tiles.iter().map(|image| {
            image.size_in_pixels(cx).filter(|(w, h)| *w > 0 && *h > 0).map_or(1.5, |(w, h)| w as f64 / h as f64)
        }).collect();
        let slots = self.rows.iter().map(|row| match row {
            Row::Gallery(urls) => urls.len(),
            _ => 0,
        }).max().unwrap_or(0);
        let height = gallery_height(self.list_width, slots, &aspects);
        for (image, aspect) in tiles.iter().zip(&aspects) {
            image.set_walk_and_fit(cx, Walk::fixed((height * aspect).round(), height.round()), ImageFit::Stretch);
        }
    }
    /// The pane's width says how many apps a row holds. Two when the line beside
    /// the art has room, one in a narrow window.
    /// The consent sheet's list has the room the window leaves it: its icon, its
    /// buttons and a margin each side stay inside.
    fn fit_sheet(&mut self, cx: &mut Cx2d) {
        let height = cx.turtle().rect().size.y;
        if !height.is_finite() || height <= 0.0 {
            return;
        }
        let room = (height - 176.0).floor().max(120.0);
        if room != self.sheet_room {
            self.sheet_room = room;
            let mut list = self.view.widget(cx, ids!(sheet_list));
            script_apply_eval!(cx, list, {max_height: #(room)});
        }
    }
    fn fit_columns(&mut self, cx: &mut Cx2d) {
        let width = cx.turtle().rect().size.x;
        if !width.is_finite() || width <= 0.0 {
            return;
        }
        self.list_width = width;
        let columns = if width >= 1180.0 { 3 } else if width >= 640.0 { 2 } else { 1 };
        if columns != self.columns {
            self.columns = columns;
            // In the list's own draw: the rows alone, nothing that redraws it.
            self.desk_rows();
        }
    }
    /// A list row's hairline stops at the last row, and a product page's words.
    fn finish_desk_entry(&mut self, cx: &mut Cx, item: &WidgetRef, entry: &Entry, index: usize) {
        let last = !matches!(self.rows.get(index + 1), Some(Row::App(_)));
        item.widget(cx, ids!(rule)).set_visible(cx, !last);
        item.label(cx, ids!(caption)).set_text(cx, if entry.kind == CatalogKind::Preview {
            "Included with OctoSense"
        } else {
            &entry.publisher
        });
        let (reason, remedy) = match &entry.status {
            EntryStatus::Unavailable(reason) => {
                let (words, remedy) = unavailable_words(reason, &HostApis::here());
                (Some(words), remedy)
            }
            // The page's banner already says why, when it has one.
            EntryStatus::Available | EntryStatus::UpdateAvailable
                if entry.consent.is_none() && self.snapshot.as_ref().is_none_or(|s| s.warning.is_none()) =>
                (Some("Getting apps is paused until App Hub has a newer catalog.".to_string()), None),
            _ => (None, None),
        };
        item.label(cx, ids!(unavailable)).set_text(cx, reason.as_deref().unwrap_or_default());
        item.widget(cx, ids!(unavailable)).set_visible(cx, reason.is_some());
        item.widget(cx, ids!(remedy)).set_visible(cx, remedy == Some(Remedy::SoftwareUpdate));
        item.widget(cx, ids!(why)).set_visible(cx, reason.is_some());
    }
    fn draw_grid(&mut self, cx: &mut Cx, item: &WidgetRef, entries: &[Entry], index: usize) {
        let last = !matches!(self.rows.get(index + 1), Some(Row::Grid(_)));
        let columns = self.columns.max(1);
        for (i, id) in [ids!(c0), ids!(c1), ids!(c2)].into_iter().enumerate() {
            let cell = item.widget(cx, id);
            cell.set_visible(cx, i < columns);
            let entry = entries.get(i);
            cell.widget(cx, ids!(row)).set_visible(cx, entry.is_some());
            cell.widget(cx, ids!(rule)).set_visible(cx, entry.is_some() && !last);
            if let Some(entry) = entry {
                self.fill_entry(cx, &cell, entry);
            }
        }
    }
    /// Press on an app: its button acts, the rest of the row opens its page.
    fn entry_press(&mut self, cx: &mut Cx, scope: &mut Scope, actions: &Actions, item: &WidgetRef, entry: Entry) {
        if item.button(cx, ids!(remedy)).clicked(actions) {
            // The shell's Software Update, as its system menu opens it.
            cx.widget_action(self.widget_uid(), AppHubAction::Launch("updater".into()));
        } else if item.button(cx, ids!(open)).clicked(actions) {
            if shows_open(&entry) && !self.busy {
                self.send(Command::Open(entry.id.clone()));
                self.render(cx);
            }
        } else if item.button(cx, ids!(action)).clicked(actions) {
            self.notice.clear();
            self.activate(cx, scope, entry);
        } else if item.widget(cx, ids!(row)).as_view().finger_up(actions).is_some_and(|up| up.was_tap()) {
            self.selected = Some(entry);
            self.confirming = false;
            self.about_full = false;
            self.notice.clear();
            self.reset_scroll(cx);
            self.render(cx);
        }
    }
    fn open_side(&mut self, cx: &mut Cx, side: Side) {
        self.update_queue = None;
        self.query.clear();
        self.view.text_input(cx, ids!(search)).set_text(cx, "");
        self.preview = side == Side::Included;
        self.about_full = false;
        self.notice.clear();
        self.page = side.page();
        self.selected = None;
        self.confirming = false;
        self.category = side.category().map(str::to_owned);
        self.reset_scroll(cx);
        self.render(cx);
    }
    /// A search leaves what was open beside it: clearing the field goes back.
    fn search_for(&mut self, cx: &mut Cx, query: String) {
        self.update_queue = None;
        self.query = query;
        self.selected = None;
        self.confirming = false;
        self.about_full = false;
        self.notice.clear();
        if self.query.trim().is_empty() {
            if self.page == Page::Search {
                self.page = self.search_home;
            }
        } else if self.page != Page::Search {
            self.search_home = self.page;
            self.page = Page::Search;
        }
        self.reset_scroll(cx);
        self.render(cx);
    }
    /// Update All, as the Mac App Store has it, except that each app still asks
    /// first: what an app may use can change with a release, and a person agrees
    /// to that app by app. Cancel stops the rest.
    fn update_all(&mut self, cx: &mut Cx) {
        if self.busy || self.installing {
            return;
        }
        self.update_queue = Some(self.updates().iter().filter(|e| can_activate(e)).map(|e| e.id.clone()).collect());
        self.next_update(cx);
        self.render(cx);
    }
    /// The next app of an Update All, or the Updates page when every one is done.
    fn next_update(&mut self, cx: &mut Cx) {
        let updates = self.updates();
        while let Some(id) = self.update_queue.as_mut().filter(|queue| !queue.is_empty()).map(|queue| queue.remove(0)) {
            if let Some(entry) = updates.iter().find(|e| e.id == id && can_activate(e)) {
                self.notice.clear();
                self.selected = Some(entry.clone());
                self.confirming = true;
                self.reset_scroll(cx);
                return;
            }
        }
        if self.update_queue.take().is_some() {
            self.notice.clear();
            self.selected = None;
            self.reset_scroll(cx);
        }
    }
    fn confirm_install(&mut self, cx: &mut Cx) {
        if self.busy || self.installing {
            return;
        }
        if let Some(consent) = self.selected.as_ref().filter(|_| self.confirming).and_then(|e| e.consent.clone()) {
            self.installing = true;
            self.send(Command::Install(consent));
            self.render(cx);
        }
    }
    fn desk_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        if self.confirming {
            if self.view.button(cx, ids!(sheet_more)).clicked(actions) {
                self.sheet_details = !self.sheet_details;
                self.show_sheet(cx);
            }
            if self.view.button(cx, ids!(sheet_confirm)).clicked(actions) {
                self.confirm_install(cx);
            }
            if self.view.button(cx, ids!(sheet_cancel)).clicked(actions) && !self.installing {
                self.confirming = false;
                // Cancel stops an Update All, back on the Updates page.
                if self.update_queue.take().is_some() {
                    self.selected = None;
                }
                self.render(cx);
            }
            return;
        }
        for (id, side) in [
            (ids!(today), Side::Discover),
            (ids!(apps), Side::Apps),
            (ids!(library), Side::Installed),
            (ids!(updates), Side::Updates),
            (ids!(cat_travel), Side::Travel),
            (ids!(cat_photo), Side::Photo),
            (ids!(cat_work), Side::Work),
            (ids!(included), Side::Included),
        ] {
            if self.view.button(cx, id).clicked(actions) {
                self.open_side(cx, side);
            }
        }
        if self.view.button(cx, ids!(back)).clicked(actions) {
            self.back(cx);
        }
        if self.view.button(cx, ids!(refresh)).clicked(actions) {
            self.refresh(cx);
            self.render(cx);
        }
        if let Some(query) = self.view.text_input(cx, ids!(search)).changed(actions) {
            self.search_for(cx, query);
        }
        if self.view.button(cx, ids!(search_clear)).clicked(actions) {
            self.view.text_input(cx, ids!(search)).set_text(cx, "");
            self.search_for(cx, String::new());
        }
        if self.view.button(cx, ids!(empty_browse)).clicked(actions) {
            self.open_side(cx, Side::Included);
        }
        if self.view.button(cx, ids!(empty_action)).clicked(actions) {
            match self.empty.as_ref().and_then(|pane| pane.action).map(|(_, action)| action) {
                Some(EmptyAction::Refresh) => {
                    self.refresh(cx);
                    self.render(cx);
                }
                Some(EmptyAction::Discover) => self.open_side(cx, Side::Discover),
                None => {}
            }
        }
        let items = self.view.portal_list(cx, ids!(list)).items_with_actions(actions);
        for (index, item) in items {
            let Some(row) = self.rows.get(index).cloned() else {
                continue;
            };
            match row {
                Row::App(entry) | Row::Detail(entry) => self.entry_press(cx, scope, actions, &item, entry),
                Row::UpdatesHead(_) if item.button(cx, ids!(update_all)).clicked(actions) => self.update_all(cx),
                Row::Grid(entries) => {
                    for (entry, id) in entries.into_iter().zip([ids!(c0), ids!(c1), ids!(c2)]) {
                        let cell = item.widget(cx, id);
                        self.entry_press(cx, scope, actions, &cell, entry);
                    }
                }
                Row::Banner(banner) if item.button(cx, ids!(retry)).clicked(actions) => match banner.retry {
                    Retry::Install => {
                        self.notice.clear();
                        self.confirming = self.selected.as_ref().is_some_and(|e| e.consent.is_some());
                        self.render(cx);
                    }
                    Retry::Catalog => {
                        self.notice.clear();
                        self.refresh(cx);
                        self.render(cx);
                    }
                    Retry::None => {}
                },
                Row::About(..) if item.button(cx, ids!(more)).clicked(actions) => {
                    self.about_full = true;
                    self.render(cx);
                }
                _ => {}
            }
        }
    }
    /// A Mac's keys: find, reload, back; Escape and Return answer the sheet.
    /// True when the key was taken.
    fn desk_key(&mut self, cx: &mut Cx, key: &KeyEvent) -> bool {
        let mods = key.modifiers;
        if self.confirming {
            match key.key_code {
                KeyCode::Escape if !self.installing => {
                    self.confirming = false;
                    self.render(cx);
                }
                KeyCode::ReturnKey | KeyCode::NumpadEnter => self.confirm_install(cx),
                _ => return false,
            }
            return true;
        }
        if mods.is_primary() && !mods.shift && !mods.alt {
            match key.key_code {
                KeyCode::KeyF => self.view.text_input(cx, ids!(search)).take_key_focus(cx),
                KeyCode::KeyR => {
                    self.refresh(cx);
                    self.render(cx);
                }
                KeyCode::LBracket => {
                    self.back(cx);
                }
                _ => return false,
            }
            return true;
        }
        if key.key_code == KeyCode::Escape && !mods.any() {
            let field = self.view.text_input(cx, ids!(search));
            if !self.query.is_empty() {
                field.set_text(cx, "");
                self.search_for(cx, String::new());
                cx.set_key_focus(Area::Empty);
                return true;
            }
            if field.key_focus(cx) {
                cx.set_key_focus(Area::Empty);
                return true;
            }
            return self.back(cx);
        }
        false
    }
}

/// What came with OctoSense and opens on this desktop: its Launchpad's apps.
fn included_entries() -> Vec<Entry> {
    catalog::desktop_entries()
}
/// Discover's sections: one per category, in the catalog's order.
fn group_by_category(entries: Vec<Entry>) -> Vec<(Option<String>, Vec<Entry>)> {
    let mut sections: Vec<(String, Vec<Entry>)> = Vec::new();
    for entry in entries {
        match sections.iter_mut().find(|(category, _)| *category == entry.category) {
            Some((_, list)) => list.push(entry),
            None => sections.push((entry.category.clone(), vec![entry])),
        }
    }
    sections.into_iter().map(|(category, list)| (Some(category_name(&category)), list)).collect()
}
/// A plain button says what it does; a disabled one dims rather than asks for a refresh.
fn desk_button_text(entry: &Entry) -> &str {
    match entry.status {
        EntryStatus::BuiltIn | EntryStatus::Installed => "Open",
        EntryStatus::UpdateAvailable => "Update",
        EntryStatus::Available => "Get",
        EntryStatus::Unavailable(_) => "Unavailable",
    }
}
/// A catalog's category slug in front of a person.
fn category_name(slug: &str) -> String {
    match slug {
        "photo-video" => "Photo & Video".into(),
        "developer" => "Developer Tools".into(),
        _ => slug
            .split(['-', '_'])
            .map(|word| {
                let mut letters = word.chars();
                letters.next().map_or(String::new(), |first| first.to_uppercase().chain(letters).collect())
            })
            .collect::<Vec<_>>()
            .join(" "),
    }
}
/// `2026-10-11` as a Mac writes it: `Oct 11, 2026`.
fn date_words(iso: &str) -> Option<String> {
    date_words_at(iso, makepad_widgets::makepad_platform::script::timer::script_local_utc_offset_secs())
}
/// "Nov 5, 2026" for a catalog published at `2026-11-06T00:12:00Z`, eight hours
/// behind UTC: a time is shown on this computer's calendar (its offset is the
/// one the shell's clock sets). A date alone stays the date it says.
fn date_words_at(iso: &str, offset_secs: i64) -> Option<String> {
    let (year, month, day) = local_date(iso, offset_secs)?;
    let name = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"].get(usize::try_from(month - 1).ok()?)?;
    Some(format!("{name} {day}, {year}"))
}
fn local_date(iso: &str, offset_secs: i64) -> Option<(i64, i64, i64)> {
    let mut parts = iso.get(..10)?.split('-');
    let (year, month, day): (i64, i64, i64) = (parts.next()?.parse().ok()?, parts.next()?.parse().ok()?, parts.next()?.parse().ok()?);
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    // `T00:12:00Z`: a UTC time. Anything else is read for its date.
    let seconds = iso.get(10..).and_then(|time| {
        let mut clock = time.strip_prefix('T')?.strip_suffix('Z')?.split(':');
        let (hour, minute, second): (i64, i64, i64) = (clock.next()?.parse().ok()?, clock.next()?.parse().ok()?, clock.next()?.parse().ok()?);
        (clock.next().is_none() && hour < 24 && minute < 60 && second < 61).then_some(hour * 3600 + minute * 60 + second)
    });
    let Some(seconds) = seconds else {
        return Some((year, month, day));
    };
    let local = days_from_civil(year, month, day) * 86_400 + seconds + offset_secs;
    Some(civil_from_days(local.div_euclid(86_400)))
}
/// Days from 1970-01-01 to a date, and back (Howard Hinnant's algorithms).
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let yoe = year - era * 400;
    let doy = (153 * ((month + 9) % 12) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(month <= 2), month, day)
}
/// An update's row: which version, and what it brings.
fn update_line(entry: &Entry) -> String {
    let notes = plain_notes(&entry.release_notes);
    if notes.is_empty() {
        format!("Version {}", entry.version)
    } else {
        format!("Version {} · {notes}", entry.version)
    }
}
/// Words that describe how an app was built or published rather than what it does.
fn has_developer_words(text: &str) -> bool {
    developer_words_at(text).is_some()
}
/// Where a text first speaks to developers: the start of its first such word or phrase.
fn developer_words_at(text: &str) -> Option<usize> {
    const WORDS: &[&str] = &[
        "octoscript", "l0", "rinx", "makepad", "contract", "oauth", "fixture", "fixtures", "attestation", "attested",
        "admission", "admits", "admitted", "runtime", "manifest", "synthetic", "soak", "immutable", "workflow", "receipt",
        "receipts", "keyless", "webassembly", "wasm", "host", "hosts", "unverified", "validate", "validated", "validation",
        "sample", "legacy", "agent", "agents", "sdk", "api", "ui", "fictional", "advertised", "authorization", "credentials",
        "identity", "publishing",
    ];
    const PHRASES: &[&str] = &[
        "developer preview", "development preview", "publisher-github", "pr #", "hub #", "app id", "app-agent", "io.github", "org.octosense",
    ];
    // ASCII lowering keeps every offset where it was; the words are all ASCII.
    let lower = text.to_ascii_lowercase();
    let phrase = PHRASES.iter().filter_map(|phrase| lower.find(phrase)).min();
    let mut start = 0;
    let mut word = None;
    for (at, c) in lower.char_indices().chain([(lower.len(), ' ')]) {
        if !c.is_alphanumeric() {
            if WORDS.contains(&&lower[start..at]) {
                word = Some(start);
                break;
            }
            start = at + c.len_utf8();
        }
    }
    phrase.into_iter().chain(word).min()
}
/// The sentences of a text that a person reads for what the app does. The
/// listing's whole text stays one press away ("More…"): this is what a page
/// opens with, not a rewrite of what the publisher signed.
fn plain_sentences(text: &str) -> String {
    let mut kept: Vec<String> = Vec::new();
    for paragraph in text.lines() {
        let mut rest = paragraph.trim();
        while !rest.is_empty() {
            let (sentence, tail) = match rest.find(". ") {
                Some(at) => (&rest[..=at], rest[at + 2..].trim_start()),
                None => (rest, ""),
            };
            if !has_developer_words(sentence) {
                kept.push(sentence.to_string());
            }
            rest = tail;
        }
    }
    kept.join(" ")
}
/// A row's one line: the publisher's, unless it is about how the app was made.
/// Then it is what the line says before that, and the category only when
/// nothing is left: a category heading above the row already says it.
fn plain_line(entry: &Entry) -> String {
    if entry.subtitle.trim().is_empty() {
        category_name(&entry.category)
    } else if has_developer_words(&entry.subtitle) {
        plain_phrase(&entry.subtitle).unwrap_or_else(|| category_name(&entry.category))
    } else {
        entry.subtitle.clone()
    }
}
/// The words of a line before its first developer's word, without the words
/// that led into it ("A static camera screen rendered from OctoScript L0").
fn plain_phrase(line: &str) -> Option<String> {
    const LEADS: &[&str] = &[
        "a", "an", "and", "as", "based", "built", "by", "for", "from", "in", "made", "of", "on", "or", "powered",
        "rendered", "the", "to", "using", "via", "with", "written",
    ];
    const TRAIL: &[char] = &[',', ';', ':', '-', '(', '[', '/', '\u{2013}', '\u{2014}'];
    let mut words: Vec<&str> = line[..developer_words_at(line)?].split_whitespace().collect();
    while let Some(last) = words.last() {
        let bare = last.trim_end_matches(TRAIL).to_lowercase();
        if bare.is_empty() || LEADS.contains(&bare.as_str()) {
            words.pop();
        } else {
            break;
        }
    }
    let phrase = words.join(" ");
    let phrase = phrase.trim_end_matches(TRAIL);
    (words.len() >= 2 && phrase.len() >= 12).then(|| phrase.to_string())
}
/// What the product page opens with.
fn plain_about(description: &str, entry: &Entry) -> String {
    let kept = plain_sentences(description);
    if !kept.is_empty() {
        return kept;
    }
    let line = plain_line(entry);
    if line == category_name(&entry.category) {
        format!("{} for OctoSense.", line)
    } else {
        sentence_case(&line)
    }
}
/// Release notes, without their version prefix (`0.2.2: …`) and their publishing history.
fn plain_notes(notes: &str) -> String {
    let notes = notes.trim();
    let body = match notes.split_once(": ") {
        Some((head, tail)) if head.chars().next().is_some_and(|c| c.is_ascii_digit()) && !head.contains(' ') => tail,
        _ => notes,
    };
    let kept = plain_sentences(body);
    if kept.is_empty() {
        return String::new();
    }
    if body == notes { kept } else { sentence_case(&kept) }
}
/// Why an app is unavailable, as a person would be told.
fn plain_reason(reason: &str) -> String {
    unavailable_words(reason, &HostApis::here()).0
}
/// What a person can do about an app App Hub can't give them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Remedy {
    /// This OctoSense lacks what the app needs; Software Update has newer ones.
    SoftwareUpdate,
}
/// The host APIs of this OctoSense, as App Hub checked an app against them.
struct HostApis {
    /// Every API it knows, at its version, with the platforms it works on.
    known: std::collections::BTreeMap<String, (u32, Vec<String>)>,
    /// What works here, the runtime's own ABIs included.
    here: std::collections::BTreeMap<String, u32>,
    platform: String,
}
impl HostApis {
    fn here() -> Self {
        use octosense_appstore::host_api;
        HostApis {
            known: host_api::methods().into_iter().map(|m| (m.name, (m.version, m.platforms))).collect(),
            here: host_api::available_versions(),
            platform: host_api::platform().into(),
        }
    }
}
/// The `name@version` APIs a refusal names: "this host does not implement
/// required APIs: charts.d3@1", "app … needs a host implementing
/// app_tools.dispatch@1".
fn needed_apis(reason: &str) -> Vec<(String, u32)> {
    reason
        .split(|c: char| c.is_whitespace() || matches!(c, ',' | '(' | ')'))
        .filter_map(|word| {
            let (name, version) = word.trim_end_matches(['.', ':', ';']).split_once('@')?;
            let version: u32 = version.parse().ok()?;
            octosense_app_policy::contract::host_api::valid_method(name).then(|| (name.to_string(), version))
        })
        .collect()
}
/// A platform as a person names it.
fn platform_words(platform: &str) -> Option<&'static str> {
    Some(match platform {
        "android" => "Android",
        "ios" => "iPhone",
        "openharmony" => "OpenHarmony",
        "macos" => "a Mac",
        "windows" => "Windows",
        "linux" => "Linux",
        _ => return None,
    })
}
/// Why an app can't be had or opened, in a person's words, and what they can
/// do about it: what the App Store says ("Requires a newer version of macOS",
/// "Not available for this Mac"), from what App Hub knows. An API this
/// OctoSense doesn't know, or knows at an older version, needs a newer
/// OctoSense; one it has at a newer version needs the app updated; one it has
/// for other platforms only works there.
fn unavailable_words(reason: &str, host: &HostApis) -> (String, Option<Remedy>) {
    let words = |text: &str| -> (String, Option<Remedy>) { (text.to_string(), None) };
    if reason == "Not offered by the current catalog" {
        return words("App Hub no longer offers this app.");
    }
    if let Some(rest) = reason.strip_prefix("component ") {
        // The app itself asks for too little: only its developer can fix that.
        if rest.contains("requires host_api.required") {
            return words("Part of this app needs a feature the app doesn’t ask for. Its developer needs to update it.");
        }
        return words("Part of this app is no longer available from App Hub. Its developer needs to update it.");
    }
    let needs = needed_apis(reason);
    if !needs.is_empty() {
        let (mut newer, mut older, mut elsewhere) = (false, false, None::<Vec<String>>);
        for (name, version) in &needs {
            let have = host.known.get(name).map(|(have, _)| *have).or_else(|| host.here.get(name).copied());
            match (have, host.known.get(name)) {
                (Some(have), _) if have > *version => older = true,
                (Some(have), Some((_, platforms))) if have == *version && !platforms.iter().any(|p| *p == host.platform) => {
                    // Where every API it needs works.
                    let mut works: Vec<String> = platforms.clone();
                    if let Some(before) = &elsewhere {
                        works.retain(|p| before.contains(p));
                    }
                    elsewhere = Some(works);
                }
                _ => newer = true,
            }
        }
        if newer {
            return ("Requires a newer version of OctoSense.".into(), Some(Remedy::SoftwareUpdate));
        }
        if older {
            return words("Doesn’t work with this version of OctoSense. Its developer needs to update it.");
        }
        let here = if host.platform == "macos" { "this Mac" } else { "this computer" };
        let works: Vec<&str> = elsewhere.unwrap_or_default().iter().filter_map(|p| platform_words(p)).collect();
        return words(&match works.as_slice() {
            [] => format!("Not available for {here}."),
            [one] => format!("Not available for {here}. It needs OctoSense on {one}."),
            [rest @ .., last] => format!("Not available for {here}. It needs OctoSense on {} or {last}.", rest.join(", ")),
        });
    }
    if reason.contains("does not provide") || reason.contains("compatible OctoSense") {
        return ("Requires a newer version of OctoSense.".into(), Some(Remedy::SoftwareUpdate));
    }
    // "org.example.notes 1.0.0 was withdrawn: <why>": the why, when it's plain.
    if let Some((_, why)) = reason.split_once(" was withdrawn: ") {
        let kept = plain_sentences(why);
        return words(&if kept.is_empty() {
            "This version was withdrawn from App Hub.".to_string()
        } else {
            format!("This version was withdrawn from App Hub. {}", sentence_case(&kept))
        });
    }
    if reason.ends_with("reinstall this app") {
        return words("This app’s files don’t match what App Hub reviewed, so it can’t open.");
    }
    if reason.starts_with("the release changed while it was opening") {
        return words("This app changed while it was opening. Try again.");
    }
    let kept = plain_sentences(reason);
    if kept.is_empty() {
        words("This app isn’t available right now.")
    } else {
        (sentence_case(&kept), None)
    }
}
/// The app's own tools by what they may do, rather than by the ids the store's
/// line lists (`githubnotes.repositories, githubnotes.files`): what each is
/// allowed, from its reviewed `tools.json`, not what its publisher calls it.
fn tool_words(tools: &[ToolUse]) -> Option<String> {
    use octosense_app_policy::Risk;
    let unattended = |risk| tools.iter().any(|tool| tool.risk == risk && !tool.approved);
    let mut uses = Vec::new();
    if unattended(Risk::Read) {
        uses.push("look things up");
    }
    if unattended(Risk::Act) {
        uses.push("make changes in the app");
    }
    if tools.iter().any(|tool| tool.approved) {
        uses.push("send, share or delete things once you approve");
    }
    let last = uses.pop()?;
    let uses = match uses.as_slice() {
        [] => last.to_string(),
        [one] => format!("{one} and {last}"),
        more => format!("{}, and {last}", more.join(", ")),
    };
    Some(format!("Its assistant can use the app’s own tools to {uses}."))
}
/// The permissions a desktop shows: the store's lines in a person's words.
fn desk_permissions(entry: &Entry) -> Vec<String> {
    let lines: Vec<String> = entry.permissions.iter().map(|line| match tool_words(&entry.tools) {
        Some(words) if line.starts_with("Its assistant can use these app tools:") => words,
        _ => line.clone(),
    }).collect();
    desk_policy(&lines)
}
/// A policy list as a desktop shows it: one short line each.
fn desk_policy(lines: &[String]) -> Vec<String> {
    plain_policy(&lines.join("\n")).lines().filter(|line| !line.is_empty()).map(str::to_string).collect()
}
/// What the privacy list adds to the permissions above it. A line that only
/// restates a permission listed there isn't said twice; every other stays.
fn desk_privacy(entry: &Entry) -> Vec<String> {
    const RESTATED: [(&str, &str); 6] = [
        ("Uses your location.", "Use your location"),
        ("Uses the camera.", "Use the camera"),
        ("Uses the clipboard.", "Use the clipboard"),
        ("May ask you questions.", "Ask you questions"),
        ("Reads your shared data.", "Read your shared data"),
        (
            "Imports or exports files and photos you choose, and can open the system sharing chooser.",
            "Import or export files and photos you choose, and open the system sharing chooser",
        ),
    ];
    desk_policy(&entry.privacy).into_iter().filter(|line| {
        !RESTATED.iter().any(|(privacy, permission)| line == privacy && entry.permissions.iter().any(|said| said == permission))
    }).collect()
}
/// A sheet's list, one line to a point. Lines past the last point share it: a
/// sheet never drops one.
fn fill_points(cx: &mut Cx, list: &WidgetRef, lines: &[String]) {
    let points = [
        ids!(b0), ids!(b1), ids!(b2), ids!(b3), ids!(b4), ids!(b5), ids!(b6), ids!(b7),
        ids!(b8), ids!(b9), ids!(b10), ids!(b11), ids!(b12), ids!(b13), ids!(b14), ids!(b15),
    ];
    let count = points.len();
    for (i, id) in points.into_iter().enumerate() {
        let point = list.widget(cx, id);
        point.set_visible(cx, i < lines.len());
        let text = if i + 1 == count && lines.len() > count {
            lines[i..].join("\n• ")
        } else {
            lines.get(i).cloned().unwrap_or_default()
        };
        point.label(cx, ids!(line)).set_text(cx, &text);
    }
}
/// How tall a row of a product's pictures is drawn: side by side, as tall as the
/// pane's width allows and never taller than a tile. A row with fewer pictures
/// than `slots` keeps the height of the fuller row above it.
fn gallery_height(width: f64, slots: usize, aspects: &[f64]) -> f64 {
    const TILE: f64 = 232.0;
    if aspects.is_empty() {
        return TILE;
    }
    let slots = slots.max(aspects.len()) as f64;
    let room = (width - 40.0 - 12.0 * (slots - 1.0)).max(1.0);
    let average = aspects.iter().sum::<f64>() / aspects.len() as f64;
    (room / (average * slots)).min(TILE)
}
/// A tool's id in a line: `notes.draft_open` reads `draft open`, without the app's
/// own name. A line about where an app connects keeps its host names.
fn plain_tool_ids(text: &str) -> String {
    const OWN: &str = "these app tools: ";
    let own = text.split_once(OWN).and_then(|(_, tools)| tools.split_once('.')).map(|(own, _)| own.to_string());
    let line = |line: &str| -> String {
        if line.starts_with("Reach only:") || line.starts_with("Declared network destinations:") {
            return line.to_string();
        }
        line.split(' ').map(|word| {
            let body = word.trim_end_matches([',', '.', ':', ';']);
            let is_id = body.contains('.')
                && body.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '-'));
            if !is_id {
                return word.to_string();
            }
            let name = own.as_deref().and_then(|own| body.strip_prefix(own)?.strip_prefix('.')).unwrap_or(body);
            format!("{}{}", name.replace(['.', '_'], " "), &word[body.len()..])
        }).collect::<Vec<_>>().join(" ")
    };
    text.lines().map(line).collect::<Vec<_>>().join("\n")
}
/// The store's own permission and privacy lines (`privacy_summary_with_tools`,
/// `permissions_summary`) are written for publishers. The same facts, in a person's
/// words; a line this doesn't know keeps its words, and a phone keeps them all.
fn plain_policy(text: &str) -> String {
    plain_tool_ids(text).replace(
            "Runs with its own local storage and quota. Capability and network declarations describe expected use; they are not permission limits.",
            "Keeps its data in its own space on this computer, with a size limit. What an app says it will use describes what it expects to do; it isn’t a limit.")
        .replace(
            "No network destinations declared; this does not mean the app is offline.",
            "It doesn’t say which sites it connects to, but it can still go online.")
        .replace("Declared network destinations:", "It says it connects to:")
        .replace("Keep its own data on this device", "Keep its own data on this computer")
        .replace(
            "Device access, connected-account scopes, external-write review and sharing with other apps still need the host's authorization.",
            "OctoSense still decides on access to your devices and connected accounts, on changes outside the app, and on sharing with other apps.")
        .replace(
            "Available host services can contact external providers after their actual authorization checks, even when a usage declaration is absent.",
            "OctoSense’s own services can contact outside providers for the app after their usual checks, even if the app doesn’t say it will.")
        .replace(
            "Network declarations do not restrict those service requests.",
            "What an app says about the sites it connects to doesn’t limit them.")
        .replace(
            "Connects its own provider accounts or signs in to its developer's backend through the host; credentials stay with the host.",
            "Connects its own accounts, or signs in to its developer’s service, through OctoSense; your sign-in details stay with OctoSense.")
        .replace(
            "Inspects the host's available APIs; this grants no access to their data or devices.",
            "Sees which OctoSense features are available; this gives it no access to their data or devices.")
        .replace(
            "Runs its own functions in a sandbox on this device; they share the app's isolation, quotas and host consent checks.",
            "Runs its own functions in a protected space on this device, under the same limits and permission checks as the app.")
        .replace(
            "Run its own sandboxed functions on this device; they reach only what the app itself may",
            "Run its own functions in a protected space on this device; they reach only what the app itself may")
        .replace(
            "Uses your signed-in Matrix identity for the Palpo operations you allow; passwords, access tokens and fleet configuration stay with the host.",
            "Uses your signed-in Matrix account for the Palpo actions you allow; passwords, sign-in details and server settings stay with OctoSense.")
        .replace("requests native host review before sending", "asks you to review in OctoSense before sending")
        .replace("request native send review", "ask you to review before sending")
        .replace("playback requires a tap", "playback requires a click")
        .replace("Reach only:", "Connect to:")
        .replace("Reach the network: nothing listed", "Connect to the internet: no sites listed")
        .replace("Inspect available host APIs without gaining access to their data or permissions", "See which OctoSense features are available, without access to their data")
        .replace("its developer’s backend", "its developer’s service")
        .replace("Runs no assistant.", "It has no assistant.")
        .replace("for its admitted tools", "for its tools")
        .replace("No app-declared background assistant or automatic triggers.", "It has no assistant in the background, and nothing that starts by itself.")
        .replace("the host's", "OctoSense’s")
        .replace("through the host", "through OctoSense")
        .replace("in the host", "in OctoSense")
        .replace("with the host", "with OctoSense")
        .replace("Shared host services", "Shared OctoSense services")
        .replace("Available host services", "OctoSense services")
        .replace("host services", "OctoSense services")
}
/// A capital first, a stop last.
fn sentence_case(text: &str) -> String {
    let text = text.trim();
    let mut letters = text.chars();
    let Some(first) = letters.next() else {
        return String::new();
    };
    let mut out: String = first.to_uppercase().chain(letters).collect();
    if !out.ends_with(['.', '!', '?', '…']) {
        out.push('.');
    }
    out
}
/// App Hub's own messages, in the banner's shape.
fn notice_banner(notice: &str) -> Banner {
    let (title, note, retry) = if let Some(rest) = notice.strip_prefix("Installation didn’t finish.") {
        let kept = plain_sentences(rest);
        ("Installation Didn’t Finish", if kept.is_empty() { "Check your connection and try again.".to_string() } else { sentence_case(&kept) }, Retry::Install)
    } else if notice.starts_with("Installed.") {
        ("Installed", "Your app is ready to open.".to_string(), Retry::None)
    } else if let Some(rest) = notice.strip_prefix("Could not open this app.") {
        ("Can’t Open This App", plain_reason(rest.trim()), Retry::None)
    } else if notice.starts_with("App Hub is unavailable") || notice.starts_with("Could not start App Hub") {
        ("App Hub Is Unavailable", "Close App Hub and open it again.".to_string(), Retry::None)
    } else {
        ("App Hub", sentence_case(&plain_sentences(notice)), Retry::None)
    };
    Banner { title: title.into(), note, retry }
}
/// What is wrong with the catalog, in words: the raw error is for logs.
fn warning_banner(warning: &str, published: Option<&str>) -> Banner {
    let updated = published.and_then(date_words).map_or(String::new(), |date| format!("Last updated {date}. "));
    let stale = warning.contains("installs pause");
    let (title, note, retry) = if warning.contains("channel changed") {
        ("Reopen App Hub", "App Hub needs to reopen before it can continue.".to_string(), Retry::None)
    } else if warning.contains("last verified cached catalog") {
        ("Can’t Connect to App Hub", format!("Showing the apps from your last connection. {updated}{}", if stale { "Getting apps is paused until App Hub reconnects." } else { "" }).trim().to_string(), Retry::Catalog)
    } else if warning.contains("could not load a verified catalog") {
        ("Can’t Connect to App Hub", "Check your connection and try again.".to_string(), Retry::Catalog)
    } else if warning.contains("cached catalog") {
        ("Can’t Read the Saved Catalog", "App Hub gets a new copy when it connects.".to_string(), Retry::Catalog)
    } else if stale {
        ("Catalog Out of Date", format!("{updated}Getting apps is paused until App Hub has a newer catalog."), Retry::Catalog)
    } else {
        ("App Hub Needs Attention", "Some of its information couldn’t be checked. Try again, or reopen App Hub.".to_string(), Retry::Catalog)
    };
    Banner { title: title.into(), note, retry }
}

fn can_activate(entry: &Entry) -> bool {
    match entry.status {
        EntryStatus::Available | EntryStatus::UpdateAvailable => entry.consent.is_some(),
        EntryStatus::Unavailable(_) => false,
        EntryStatus::BuiltIn | EntryStatus::Installed => true,
    }
}
/// An update on offer leaves the installed release openable: the detail
/// card offers Open beside Update. An installed app's own button says Open.
fn shows_open(entry: &Entry) -> bool {
    entry.status == EntryStatus::UpdateAvailable && entry.can_open
}
fn button_text(entry: &Entry) -> &str {
    if matches!(
        entry.status,
        EntryStatus::Available | EntryStatus::UpdateAvailable
    ) && entry.consent.is_none()
    {
        return "Refresh";
    }
    match entry.status {
        EntryStatus::BuiltIn | EntryStatus::Installed => "Open",
        EntryStatus::UpdateAvailable => "Update",
        EntryStatus::Available => "Get",
        EntryStatus::Unavailable(_) => "Unavailable",
    }
}
fn lines_or(lines: &[String], empty: &str) -> String {
    if lines.is_empty() {
        empty.into()
    } else {
        lines.join("\n\n")
    }
}
impl Widget for AppHubView {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.start(cx);
        self.drain(cx, scope);
        // Its own agent's tool calls, answered from the catalog this view
        // shows (read tools only); the link is set aside for the answer.
        if let Some(mut agent) = self.agent.take() {
            let snapshot = self.snapshot.as_ref();
            agent.serve_tools(cx, event, |call| crate::ai::answer(snapshot, call));
            self.agent = Some(agent);
        }
        if let Event::NetworkResponses(responses) = event {
            self.receive_svg(cx, responses);
            handle_image_cache_network_responses(cx, responses);
        }
        if let Event::BackPressed { handled } = event {
            if !handled.get() && self.back(cx) {
                handled.set(true);
            }
        }
        if self.desk {
            if let Event::KeyDown(key) = event {
                if self.desk_key(cx, key) {
                    return;
                }
            }
            if self.spin.is_event(event).is_some() && self.busy {
                self.view.widget(cx, ids!(busy_mark)).redraw(cx);
                self.view.widget(cx, ids!(empty_busy)).redraw(cx);
            }
        }
        if let Event::Actions(actions) = event {
            self.actions(cx, actions, scope);
        }
        self.view.handle_event(cx, event, scope);
    }
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.start(cx);
        if self.style_changed {
            self.render(cx);
        }
        if self.desk && self.busy {
            self.spin = cx.new_next_frame();
        }
        if self.desk {
            self.fit_sheet(cx);
        }
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                self.draw_rows(cx, &mut list);
            }
        }
        DrawStep::done()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use makepad_app_module::AppModule;
    use makepad_widgets::makepad_platform::makepad_network::{
        EventSink, NetworkBackend, NetworkError, NetworkRuntime, WsSend,
    };

    /// Presentation tests supply their own responses. Record the request rather
    /// than starting a detached native HTTPS worker that can outlive the test.
    #[derive(Default)]
    struct RecordingNetwork {
        requests: Mutex<Vec<(LiveId, HttpRequest)>>,
    }
    impl NetworkBackend for RecordingNetwork {
        fn http_start(&self, id: LiveId, request: HttpRequest, _: EventSink) -> Result<(), NetworkError> {
            self.requests.lock().unwrap().push((id, request));
            Ok(())
        }
        fn http_cancel(&self, _: LiveId) -> Result<(), NetworkError> { Ok(()) }
        fn ws_open(&self, _: LiveId, _: HttpRequest, _: EventSink) -> Result<(), NetworkError> {
            Err(NetworkError::Unsupported("presentation tests do not use WebSockets"))
        }
        fn ws_send(&self, _: LiveId, _: WsSend) -> Result<(), NetworkError> {
            Err(NetworkError::Unsupported("presentation tests do not use WebSockets"))
        }
        fn ws_close(&self, _: LiveId) -> Result<(), NetworkError> {
            Err(NetworkError::Unsupported("presentation tests do not use WebSockets"))
        }
    }

    #[test]
    fn every_included_app_says_what_it_does_in_its_own_words() {
        // A line with a developer's word would fall back to the category name.
        for entry in catalog::desktop_entries() {
            assert_eq!(plain_line(&entry), entry.subtitle, "{}", entry.id);
            // About keeps every sentence; it joins paragraphs with a space.
            let words = |text: &str| text.split_whitespace().collect::<Vec<_>>().join(" ");
            assert_eq!(words(&plain_about(&entry.description, &entry)), words(&entry.description), "{}", entry.id);
        }
    }

    #[test]
    fn completed_install_survives_closed_store_view() {
        let (sender, receiver) = mpsc::channel();
        drop(receiver); // AppHubView::shutdown drops this while work is in flight.
        assert!(!publish_reply(&sender, Reply::Installed {
            app_id: "org.example.failed".into(),
            snapshot: Default::default(),
            error: Some("Download failed".into()),
        }));
        assert!(take_completed_installs().is_empty());
        assert!(!publish_reply(&sender, Reply::Installed {
            app_id: "org.example.updated".into(),
            snapshot: Default::default(),
            error: None,
        }));
        assert_eq!(take_completed_installs(), vec!["org.example.updated"]);
        assert!(take_completed_installs().is_empty(), "each completion is consumed once");
    }
    #[test]
    fn native_module_and_all_presentation_states_evaluate() {
        let mut cx = Cx::new(Box::new(|_, _| {}));
        let network = Arc::new(RecordingNetwork::default());
        cx.net = Arc::new(NetworkRuntime::with_backend(network.clone()));
        cx.with_vm(makepad_widgets::script_mod);
        let vm_id = cx.alloc_splash_vm_with_network(false);
        let root = cx.with_script_vm_id_trusted(vm_id, |vm| {
            crate::APP_HUB_MODULE.register(vm);
            let value = script_eval!(vm,{use mod.widgets.* AppHubView{}});
            let root = WidgetRef::script_from_value(vm, value);
            let errors = vm.take_errors();
            assert!(errors.is_empty(), "native App Hub errors: {errors:?}");
            root
        });
        let entered = makepad_widgets::widget_async::enter_isolate(&mut cx, vm_id);
        let artwork = cx.with_vm(|vm| {
            let value = script_eval!(vm, {use mod.widgets.* Image{}});
            WidgetRef::script_from_value(vm, value).as_image()
        });
        {
            let mut view = root.borrow_mut::<AppHubView>().unwrap();
            view.started = true; // Presentation tests must not fetch a real catalog.
            // Both store surfaces must render the shared launcher identity,
            // not a second set of toolbar glyphs or category-colored artwork.
            for template in [id!(Row), id!(Detail)] {
                let item = view.view.portal_list(&mut cx, ids!(list)).item(&mut cx, 0, template);
                for entry in catalog::preview_entries() {
                    view.fill_entry(&mut cx, &item, &entry);
                    assert!(item.widget(&mut cx, ids!(icon.builtin.art))
                        .borrow::<app_icon::AppIcon>().is_some(), "preview must use the shared AppIcon");
                    assert!(item.widget(&mut cx, ids!(icon.builtin)).visible());
                    let mut live = entry.clone();
                    live.kind = CatalogKind::Live;
                    view.fill_entry(&mut cx, &item, &live);
                    assert!(!item.widget(&mut cx, ids!(icon.builtin)).visible(),
                        "a recycled live listing must not retain the builtin icon");
                }
            }
            let art_dir = tempfile::tempdir().unwrap();
            let svg = art_dir.path().join("icon.svg");
            std::fs::write(&svg, br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M2 2H22V22H2Z" fill="#00ac96"/></svg>"##).unwrap();
            view.bind_image(&mut cx, artwork.clone(), svg.to_str());
            assert!(artwork.has_content(), "catalog SVG icons must render");
            let remote_svg = "https://example.com/icon.svg?revision=2";
            view.bind_image(&mut cx, artwork.clone(), Some(remote_svg));
            assert!(
                !artwork.has_content(),
                "a recycled icon must not show old artwork"
            );
            let request_id = *view.svg_requests.keys().next().unwrap();
            {
                let requests = network.requests.lock().unwrap();
                assert_eq!(requests.len(), 1, "one bounded SVG download is requested");
                assert_eq!(requests[0].0, request_id);
                assert_eq!(requests[0].1.url, remote_svg);
                assert!(matches!(requests[0].1.method, HttpMethod::GET));
                assert_eq!(requests[0].1.max_response_body_bytes, 1024 * 1024);
            }
            view.receive_svg(
                &mut cx,
                &vec![NetworkResponse::HttpResponse {
                    request_id,
                    response: HttpResponse::new(
                        LiveId(0),
                        200,
                        Default::default(),
                        Some(std::fs::read(&svg).unwrap()),
                    ),
                }],
            );
            view.bind_image(&mut cx, artwork.clone(), Some(remote_svg));
            assert!(view.svg_requests.is_empty(), "the simulated response completes the request");
            assert_eq!(network.requests.lock().unwrap().len(), 1, "cached SVGs are not downloaded again");
            assert!(
                artwork.has_content(),
                "downloaded SVGs must use the vector parser"
            );
            view.snapshot = Some(CatalogSnapshot {
                verified: true,
                ..Default::default()
            });
            view.render(&mut cx);
            assert!(view.rows.iter().any(|row| matches!(row, Row::Empty(..))));
            view.preview = true;
            for page in [Page::Today, Page::Apps, Page::Search, Page::Library] {
                view.navigate(&mut cx, page);
                assert!(view.rows.iter().any(|row| matches!(row, Row::App(_))));
            }
            view.query = "NO MATCH".into();
            view.page = Page::Search;
            view.render(&mut cx);
            assert!(!view.rows.iter().any(|row| matches!(row, Row::App(_))));
            let entry = catalog::preview_entries().remove(0);
            view.selected = Some(entry);
            view.render(&mut cx);
            assert!(view.rows.iter().any(|row| matches!(row, Row::Detail(_))));
            assert!(view.back(&mut cx));
            assert!(view.selected.is_none());
            view.preview = false;
            view.page = Page::Library;
            view.render(&mut cx);
            assert!(
                view.rows
                    .iter()
                    .any(|row| matches!(row,Row::Empty(title,_) if title==if view.desk { "No Apps Installed" } else { "Make yourself at home" })),
                "saved search must not filter Library"
            );
            let mut old = catalog::preview_entries().remove(0);
            old.kind = CatalogKind::Live;
            old.status = EntryStatus::Available;
            view.selected = Some(old.clone());
            view.confirming = true;
            view.installing = true;
            let mut changed = old;
            changed.status = EntryStatus::Unavailable("Withdrawn".into());
            let (sender, receiver) = mpsc::channel();
            view.rx = Some(receiver);
            sender
                .send(Reply::Installed {
                    app_id: changed.id.clone(),
                    snapshot: CatalogSnapshot {
                        entries: vec![changed],
                        verified: true,
                        ..Default::default()
                    },
                    error: Some("Withdrawn".into()),
                })
                .unwrap();
            view.drain(&mut cx, &mut Scope::empty());
            assert!(matches!(
                view.selected.as_ref().unwrap().status,
                EntryStatus::Unavailable(_)
            ));
            assert!(!view.confirming && !view.installing);
        }
        makepad_widgets::widget_async::leave_isolate(&mut cx, entered);
        drop(root);
        cx.free_splash_vm(vm_id);
    }
    #[test]
    fn update_all_asks_for_each_app_in_turn_and_cancel_stops_it() {
        let mut cx = Cx::new(Box::new(|_, _| {}));
        cx.with_vm(makepad_widgets::script_mod);
        let vm_id = cx.alloc_splash_vm_with_network(false);
        let root = cx.with_script_vm_id_trusted(vm_id, |vm| {
            crate::APP_HUB_MODULE.register(vm);
            let value = script_eval!(vm, {use mod.widgets.* AppHubView{}});
            let root = WidgetRef::script_from_value(vm, value);
            assert!(vm.take_errors().is_empty());
            root
        });
        let entered = makepad_widgets::widget_async::enter_isolate(&mut cx, vm_id);
        {
            let mut view = root.borrow_mut::<AppHubView>().unwrap();
            view.started = true; // No catalog request or personal profile.
            view.desk = true;
            let mut entries = catalog::preview_entries();
            let (mut a, mut b) = (entries.remove(0), entries.remove(0));
            for entry in [&mut a, &mut b] {
                entry.kind = CatalogKind::Live;
                entry.status = EntryStatus::UpdateAvailable;
                entry.consent = Some(catalog::InstallConsent::for_view_tests(&entry.id, "2.0.0"));
            }
            let installed = |entry: &Entry| Entry { status: EntryStatus::Installed, consent: None, ..entry.clone() };
            let library = |library: Vec<Entry>| CatalogSnapshot { library, verified: true, ..Default::default() };
            view.snapshot = Some(library(vec![a.clone(), b.clone()]));
            view.page = Page::Updates;
            view.render(&mut cx);
            assert!(view.rows.iter().any(|row| matches!(row, Row::UpdatesHead(true))), "Update All heads the list");
            let (sender, receiver) = mpsc::channel();
            view.rx = Some(receiver);
            let shown = |view: &AppHubView| view.selected.as_ref().filter(|_| view.confirming).map(|e| e.id.clone());

            view.update_all(&mut cx);
            assert_eq!(shown(&view), Some(a.id.clone()), "the first app asks first");
            sender.send(Reply::Installed { app_id: a.id.clone(), snapshot: library(vec![installed(&a), b.clone()]), error: None }).unwrap();
            view.drain(&mut cx, &mut Scope::empty());
            assert_eq!(shown(&view), Some(b.id.clone()), "a finished update brings the next app's own sheet");
            assert!(view.back(&mut cx), "Escape, as Cancel, closes the sheet");
            assert!(view.selected.is_none() && view.update_queue.is_none(), "and stops the run, on the Updates page");

            view.update_all(&mut cx);
            assert_eq!(shown(&view), Some(b.id.clone()));
            sender.send(Reply::Installed { app_id: b.id.clone(), snapshot: library(vec![installed(&a), installed(&b)]), error: None }).unwrap();
            view.drain(&mut cx, &mut Scope::empty());
            assert!(view.selected.is_none() && !view.confirming && view.update_queue.is_none(), "every update done: back on the Updates page");
            assert!(!view.rows.iter().any(|row| matches!(row, Row::UpdatesHead(_))), "nothing left to update");

            view.snapshot = Some(library(vec![a.clone(), b.clone()]));
            view.update_all(&mut cx);
            sender.send(Reply::Installed { app_id: a.id.clone(), snapshot: library(vec![a.clone(), b.clone()]), error: Some("The download failed.".into()) }).unwrap();
            view.drain(&mut cx, &mut Scope::empty());
            assert!(!view.confirming && view.update_queue.is_none(), "a failed update stops the run where it can be read");
            assert!(view.notice.starts_with("Installation didn’t finish."), "{}", view.notice);
        }
        makepad_widgets::widget_async::leave_isolate(&mut cx, entered);
        drop(root);
        cx.free_splash_vm(vm_id);
    }
    #[test]
    fn unavailable_detail_explains_refusal_and_clears_recycled_text() {
        let mut cx = Cx::new(Box::new(|_, _| {}));
        cx.with_vm(makepad_widgets::script_mod);
        let vm_id = cx.alloc_splash_vm_with_network(false);
        let root = cx.with_script_vm_id_trusted(vm_id, |vm| {
            crate::APP_HUB_MODULE.register(vm);
            let value = script_eval!(vm, {use mod.widgets.* AppHubView{}});
            let root = WidgetRef::script_from_value(vm, value);
            assert!(vm.take_errors().is_empty());
            root
        });
        let entered = makepad_widgets::widget_async::enter_isolate(&mut cx, vm_id);
        {
            let mut view = root.borrow_mut::<AppHubView>().unwrap();
            view.started = true; // No catalog request or personal profile.
            let detail = view.view.portal_list(&mut cx, ids!(list)).item(&mut cx, 0, id!(Detail));
            let mut entry = catalog::preview_entries().remove(0);
            entry.kind = CatalogKind::Live;
            for reason in [
                "This host does not provide app_tools.dispatch@1. Install a compatible OctoSense release.",
                "This app was withdrawn by its publisher.\nIts saved data remains on this device.",
            ] {
                entry.status = EntryStatus::Unavailable(reason.into());
                view.fill_entry(&mut cx, &detail, &entry);
                assert!(detail.widget(&mut cx, ids!(unavailable)).visible());
                assert_eq!(detail.label(&mut cx, ids!(unavailable)).text(), reason);
                assert!(!detail.widget(&mut cx, ids!(action)).borrow::<Button>().unwrap().enabled());
                assert!(!can_activate(&entry));
            }
            entry.status = EntryStatus::Installed;
            view.fill_entry(&mut cx, &detail, &entry);
            assert!(!detail.widget(&mut cx, ids!(unavailable)).visible());
            assert_eq!(detail.label(&mut cx, ids!(unavailable)).text(), "");
            assert!(detail.widget(&mut cx, ids!(action)).borrow::<Button>().unwrap().enabled());
        }
        makepad_widgets::widget_async::leave_isolate(&mut cx, entered);
        drop(root);
        cx.free_splash_vm(vm_id);
    }

    #[test]
    fn an_app_it_cannot_get_says_why_and_what_to_do() {
        let host = HostApis {
            known: [
                ("charts.d3", 1, &["macos", "android"][..]),
                ("phone.call", 1, &["android"][..]),
                ("sms.send", 1, &["android", "ios"][..]),
                ("camera.capture", 2, &["android", "macos"][..]),
            ]
            .into_iter()
            .map(|(name, version, platforms)| (name.to_string(), (version, platforms.iter().map(|p| p.to_string()).collect())))
            .collect(),
            here: [("charts.d3", 1), ("camera.capture", 2), ("app_policy.device_consent", 1)]
                .into_iter()
                .map(|(name, version)| (name.to_string(), version))
                .collect(),
            platform: "macos".into(),
        };
        let newer = ("Requires a newer version of OctoSense.".to_string(), Some(Remedy::SoftwareUpdate));
        // An API this OctoSense doesn't know, or knows at an older version.
        assert_eq!(unavailable_words("this host does not implement required APIs: sky.stars@1", &host), newer);
        assert_eq!(unavailable_words("this host does not implement required APIs: camera.capture@3, charts.d3@1", &host), newer);
        assert_eq!(unavailable_words("app org.example.radar needs a host implementing app_tools.dispatch@1", &host), newer);
        // One it has at a newer version: the app is behind.
        assert_eq!(
            unavailable_words("this host does not implement required APIs: camera.capture@1", &host),
            ("Doesn’t work with this version of OctoSense. Its developer needs to update it.".to_string(), None)
        );
        // Ones it has for other platforms: where all of them work.
        assert_eq!(
            unavailable_words("this host does not implement required APIs: phone.call@1", &host).0,
            "Not available for this Mac. It needs OctoSense on Android."
        );
        assert_eq!(
            unavailable_words("this host does not implement required APIs: sms.send@1", &host).0,
            "Not available for this Mac. It needs OctoSense on Android or iPhone."
        );
        assert_eq!(
            unavailable_words("this host does not implement required APIs: phone.call@1, sms.send@1", &host).0,
            "Not available for this Mac. It needs OctoSense on Android."
        );
        let linux = HostApis { platform: "linux".into(), ..host };
        assert_eq!(
            unavailable_words("this host does not implement required APIs: phone.call@1", &linux).0,
            "Not available for this computer. It needs OctoSense on Android."
        );
        // The parts it pins, and its own release.
        assert_eq!(
            unavailable_words("component markdown (org.example.markdown 1.0.0) was withdrawn: unsafe parser", &linux),
            ("Part of this app is no longer available from App Hub. Its developer needs to update it.".to_string(), None)
        );
        assert_eq!(
            unavailable_words("component net (org.example.net 1.0.0) requires host_api.required wasm.sockets@1 (and requires host-api-v1)", &linux).0,
            "Part of this app needs a feature the app doesn’t ask for. Its developer needs to update it."
        );
        assert_eq!(
            unavailable_words("org.example.notes 1.0.0 was withdrawn: It lost its data on update.", &linux).0,
            "This version was withdrawn from App Hub. It lost its data on update."
        );
        assert_eq!(
            unavailable_words("org.example.notes 1.0.0 was withdrawn: Legacy sample retired.", &linux).0,
            "This version was withdrawn from App Hub."
        );
        assert_eq!(
            unavailable_words("the installed manifest is not the reviewed one; reinstall this app", &linux).0,
            "This app’s files don’t match what App Hub reviewed, so it can’t open."
        );
        assert_eq!(unavailable_words("Not offered by the current catalog", &linux).0, "App Hub no longer offers this app.");
        // None of it is in a developer's words.
        for reason in [
            "this host does not implement required APIs: sky.stars@1",
            "this host does not implement required APIs: camera.capture@1",
            "this host does not implement required APIs: phone.call@1",
            "component markdown (org.example.markdown 1.0.0) is not in the catalog",
            "org.example.notes 1.0.0 was withdrawn: Legacy sample retired.",
            "the release changed while it was opening; try again",
        ] {
            let (words, _) = unavailable_words(reason, &linux);
            assert!(!has_developer_words(&words) && !words.contains('@') && !words.contains("org."), "{reason}: {words}");
        }
    }

    #[test]
    fn a_desktop_page_says_why_and_offers_software_update() {
        let mut cx = Cx::new(Box::new(|_, _| {}));
        cx.with_vm(makepad_widgets::script_mod);
        let vm_id = cx.alloc_splash_vm_with_network(false);
        let root = cx.with_script_vm_id_trusted(vm_id, |vm| {
            crate::APP_HUB_MODULE.register(vm);
            let value = script_eval!(vm, {use mod.widgets.* AppHubView{}});
            let root = WidgetRef::script_from_value(vm, value);
            assert!(vm.take_errors().is_empty());
            root
        });
        let entered = makepad_widgets::widget_async::enter_isolate(&mut cx, vm_id);
        {
            let mut view = root.borrow_mut::<AppHubView>().unwrap();
            view.started = true; // No catalog request or personal profile.
            assert!(view.desk, "a test's style is a desktop one");
            let detail = view.view.portal_list(&mut cx, ids!(list)).item(&mut cx, 0, id!(Detail));
            let mut entry = catalog::preview_entries().remove(0);
            entry.kind = CatalogKind::Live;
            // No OctoSense has this API: a newer one may.
            entry.status = EntryStatus::Unavailable("this host does not implement required APIs: fixture.future@1".into());
            view.fill_entry(&mut cx, &detail, &entry);
            view.finish_desk_entry(&mut cx, &detail, &entry, 0);
            assert!(detail.widget(&mut cx, ids!(why)).visible());
            assert_eq!(detail.label(&mut cx, ids!(unavailable)).text(), "Requires a newer version of OctoSense.");
            assert!(detail.widget(&mut cx, ids!(remedy)).visible());
            assert_eq!(detail.widget(&mut cx, ids!(remedy)).text(), "Software Update…");
            assert!(!detail.widget(&mut cx, ids!(action)).borrow::<Button>().unwrap().enabled());
            // A reason Software Update can't help with offers no way there.
            entry.status = EntryStatus::Unavailable("Not offered by the current catalog".into());
            view.fill_entry(&mut cx, &detail, &entry);
            view.finish_desk_entry(&mut cx, &detail, &entry, 0);
            assert_eq!(detail.label(&mut cx, ids!(unavailable)).text(), "App Hub no longer offers this app.");
            assert!(!detail.widget(&mut cx, ids!(remedy)).visible());
            // A recycled page keeps neither.
            entry.status = EntryStatus::Installed;
            view.fill_entry(&mut cx, &detail, &entry);
            view.finish_desk_entry(&mut cx, &detail, &entry, 0);
            assert!(!detail.widget(&mut cx, ids!(why)).visible());
            assert!(!detail.widget(&mut cx, ids!(remedy)).visible());
        }
        makepad_widgets::widget_async::leave_isolate(&mut cx, entered);
        drop(root);
        cx.free_splash_vm(vm_id);
    }

    #[test]
    fn an_assistants_tools_are_said_by_what_they_may_do() {
        use octosense_app_policy::Risk;
        let read = ToolUse { risk: Risk::Read, approved: false };
        let act = ToolUse { risk: Risk::Act, approved: false };
        let send = ToolUse { risk: Risk::Destructive, approved: true };
        assert_eq!(tool_words(&[]), None);
        assert_eq!(tool_words(&[read, read, read]).as_deref(), Some("Its assistant can use the app’s own tools to look things up."));
        assert_eq!(
            tool_words(&[read, act]).as_deref(),
            Some("Its assistant can use the app’s own tools to look things up and make changes in the app.")
        );
        assert_eq!(
            tool_words(&[send, act, read]).as_deref(),
            Some("Its assistant can use the app’s own tools to look things up, make changes in the app, and send, share or delete things once you approve.")
        );
        let mut entry = catalog::preview_entries().remove(0);
        entry.permissions = vec![
            "Run an assistant for this app, only after you allow it".into(),
            "Its assistant can use these app tools: githubnotes.repositories, githubnotes.files, githubnotes.read".into(),
        ];
        entry.tools = vec![read, read, read];
        assert_eq!(
            desk_permissions(&entry),
            ["Run an assistant for this app, only after you allow it", "Its assistant can use the app’s own tools to look things up."]
        );
        // Without the tools' own facts the line keeps their names, in words.
        entry.tools.clear();
        assert_eq!(desk_permissions(&entry)[1], "Its assistant can use these app tools: repositories, files, read");
    }

    #[test]
    fn missing_consent_and_unavailable_apps_cannot_activate() {
        let mut entry = catalog::preview_entries().remove(0);
        assert!(can_activate(&entry));
        entry.kind = CatalogKind::Live;
        entry.status = EntryStatus::Available;
        assert!(!can_activate(&entry));
        assert_eq!(button_text(&entry), "Refresh");
        entry.status = EntryStatus::Unavailable("Withdrawn".into());
        assert!(!can_activate(&entry));
    }

    #[test]
    fn an_update_on_offer_shows_open_beside_update() {
        let mut entry = catalog::preview_entries().remove(0);
        entry.kind = CatalogKind::Live;
        entry.status = EntryStatus::UpdateAvailable;
        entry.can_open = true;
        assert!(shows_open(&entry), "the installed release can be opened while the update waits");
        entry.can_open = false;
        assert!(!shows_open(&entry), "a release that cannot open offers only the update");
        entry.status = EntryStatus::Installed;
        entry.can_open = true;
        assert!(!shows_open(&entry), "an installed app's own button already says Open");
    }

    #[test]
    fn a_desktop_says_it_in_words_a_person_uses() {
        assert_eq!(category_name("photo-video"), "Photo & Video");
        assert_eq!(category_name("travel"), "Travel");
        assert_eq!(date_words("2026-10-11").as_deref(), Some("Oct 11, 2026"));
        assert_eq!(date_words("nonsense"), None);
        // A catalog's time is the day it was here; a date alone is its date.
        let pacific = -8 * 3600;
        assert_eq!(date_words_at("2026-11-06T00:12:00Z", pacific).as_deref(), Some("Nov 5, 2026"));
        assert_eq!(date_words_at("2026-11-06T00:12:00Z", 0).as_deref(), Some("Nov 6, 2026"));
        assert_eq!(date_words_at("2026-11-06T16:30:00Z", 8 * 3600).as_deref(), Some("Nov 7, 2026"));
        assert_eq!(date_words_at("2026-12-31T23:30:00Z", 3600).as_deref(), Some("Jan 1, 2027"));
        assert_eq!(date_words_at("2026-03-01T03:00:00Z", pacific).as_deref(), Some("Feb 28, 2026"));
        assert_eq!(date_words_at("2026-11-06", pacific).as_deref(), Some("Nov 6, 2026"));
        assert_eq!(date_words_at("2026-11-06T25:00:00Z", pacific).as_deref(), Some("Nov 6, 2026"), "a time it can't read leaves the date");
        for days in [-1, 0, 59, 20_398, 2_932_896] {
            let (y, m, d) = civil_from_days(days);
            assert_eq!(days_from_civil(y, m, d), days);
        }
        let mut entry = catalog::preview_entries().remove(0);
        entry.category = "photo-video".into();
        entry.subtitle = "A static camera screen rendered from OctoScript L0".into();
        assert_eq!(plain_line(&entry), "A static camera screen", "how an app was made is not its description");
        entry.subtitle = "Rendered from OctoScript L0".into();
        assert_eq!(plain_line(&entry), "Photo & Video", "nothing is left, so the category says it");
        entry.subtitle = "Trail maps for the weekend (developer preview)".into();
        assert_eq!(plain_line(&entry), "Trail maps for the weekend");
        entry.subtitle = "A personal watchlist for open-source projects".into();
        assert_eq!(plain_line(&entry), "A personal watchlist for open-source projects");
        assert_eq!(
            plain_sentences("Search public repositories. It uses its own Rust WebAssembly component. Requires a compatible OctoSense host.\nNo account is required."),
            "Search public repositories. No account is required."
        );
        assert_eq!(
            plain_notes("0.2.1: redesign sign-in. Live GitHub OAuth remains unverified."),
            "Redesign sign-in."
        );
        assert_eq!(plain_notes("Compact watchlist and charts."), "Compact watchlist and charts.");
        assert_eq!(
            plain_reason("This host does not provide app_tools.dispatch@1. Install a compatible OctoSense release."),
            "Requires a newer version of OctoSense."
        );
        assert_eq!(
            plain_reason("This app was withdrawn by its publisher.\nIts saved data remains on this device."),
            "This app was withdrawn by its publisher. Its saved data remains on this device."
        );
        assert_eq!(plain_reason("Legacy sample retired. Install io.github.ymote.notes from App Hub."), "This app isn’t available right now.");
        let stale = warning_banner("the catalog is 22 days old; installs pause until the hub is reachable again", Some("2026-10-11"));
        assert_eq!(stale.title, "Catalog Out of Date");
        assert!(stale.note.starts_with("Last updated Oct 11, 2026."), "{}", stale.note);
        assert!(stale.retry == Retry::Catalog, "a failure offers Try Again");
        let offline = warning_banner("Showing the last verified cached catalog. error sending request for url (https://example.invalid)", None);
        assert_eq!(offline.title, "Can’t Connect to App Hub");
        assert!(!offline.note.contains("example.invalid"), "a raw error stays out of the window");
        assert!(notice_banner("Installation didn’t finish. Withdrawn").retry == Retry::Install);
        // The store's own lines, with the facts kept and the publisher's words gone.
        let manifest = octosense_app_policy::AppManifest::parse(
            &serde_json::json!({"schema": 1, "id": "plain-words", "version": "1.0.0", "name": "Plain Words",
                "integrity": {"bundle_blake3": "0".repeat(64)}, "capabilities": ["storage"]}).to_string(),
        ).unwrap();
        let shown = plain_policy(&octosense_app_policy::privacy_summary(&manifest).join("\n"));
        for word in ["declaration", "declared", "Capability", "host", "scopes", "authorization"] {
            assert!(!shown.contains(word), "{word} in: {shown}");
        }
        assert!(shown.contains("it isn’t a limit"), "{shown}");
        assert!(shown.contains("it can still go online"), "{shown}");
        assert!(shown.contains("OctoSense still decides"), "{shown}");
        assert_eq!(
            plain_policy("Its assistant can use these app tools: notes.files, notes.draft_open\nYou approve every call of notes.files yourself: no standing rule can.\nIts assistant requests these additional tools: mail.send\nReach only: api.example.com, b.example.org"),
            "Its assistant can use these app tools: files, draft open\nYou approve every call of files yourself: no standing rule can.\nIts assistant requests these additional tools: mail send\nConnect to: api.example.com, b.example.org"
        );
        // Every service whose line a person could trip on, from the store's own text.
        let services = octosense_app_policy::AppManifest::parse(
            &serde_json::json!({"schema": 1, "id": "plain-services", "version": "1.0.0", "name": "Plain Services",
                "integrity": {"bundle_blake3": "0".repeat(64)}, "capabilities": ["auth", "runtime", "gmail", "youtube", "wasm"]}).to_string(),
        ).unwrap();
        let shown = plain_policy(&octosense_app_policy::privacy_summary(&services).join("\n"));
        for word in ["declaration", "host", "backend", "APIs", "sandbox", "quotas", "native", "a tap"] {
            assert!(!shown.contains(word), "{word} in: {shown}");
        }
        for fact in ["doesn’t limit them", "your sign-in details stay with OctoSense", "gives it no access", "protected space", "review in OctoSense before sending", "requires a click"] {
            assert!(shown.contains(fact), "{fact} missing from: {shown}");
        }
        assert_eq!(
            plain_policy("Read authorized Gmail messages, keep reply drafts and request native send review"),
            "Read authorized Gmail messages, keep reply drafts and ask you to review before sending",
        );
        assert!(plain_policy("Run its own sandboxed functions on this device; they reach only what the app itself may").contains("protected space"));
        assert_eq!(desk_policy(&["Use the camera".into(), "Use the microphone".into()]), ["Use the camera", "Use the microphone"]);
        assert!(desk_policy(&[]).is_empty());
    }

    #[test]
    fn a_privacy_line_that_only_repeats_a_permission_is_not_said_twice() {
        let mut entry = included_entries().remove(0);
        entry.permissions = vec!["Use the camera".into()];
        entry.privacy = vec![
            "Uses the camera.".into(),
            // No permission above says this one, so it stays.
            "Uses your location.".into(),
            "Records sound after your permission while the app is active.".into(),
        ];
        assert_eq!(desk_privacy(&entry), ["Uses your location.", "Records sound after your permission while the app is active."]);
    }

    #[test]
    fn a_products_pictures_sit_side_by_side_and_fit_the_pane() {
        // Two wide pictures in a wide pane: a whole tile's height.
        assert_eq!(gallery_height(900.0, 2, &[1.5, 1.5]), 232.0);
        // Three in a narrow pane shrink until all of them fit.
        let height = gallery_height(480.0, 3, &[1.5, 1.5, 1.5]);
        assert!(height * 4.5 + 24.0 + 40.0 <= 480.0 + 0.001, "{height}");
        // A row's one picture keeps the height of the full row above it.
        assert_eq!(gallery_height(480.0, 2, &[1.5]), gallery_height(480.0, 2, &[1.5, 1.5]));
        assert_eq!(gallery_height(0.0, 0, &[]), 232.0);
    }

    #[test]
    fn a_desktop_lists_only_what_opens_and_never_offers_a_dead_button() {
        assert!(included_entries().iter().all(|entry| entry.id != "camera"));
        let mut entry = catalog::preview_entries().remove(0);
        entry.kind = CatalogKind::Live;
        entry.status = EntryStatus::Available;
        assert_eq!(desk_button_text(&entry), "Get", "a paused catalog dims Get rather than asking for Refresh");
        assert!(!can_activate(&entry));
        let sections = group_by_category(catalog::preview_entries());
        assert_eq!(sections[0].0.as_deref(), Some("News"));
        assert!(sections.iter().any(|(title, _)| title.as_deref() == Some("Photo & Video")));
    }
}
