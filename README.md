# GtkUI

A language designed to improve readability and the overall efficiency of Gtk app development.

This language is heavily inspired by Apple's SwiftUI syntax and is generally the inspiration for this project.

## What does this solve?

Lets take a look at what the tradition "Builder File" looks like.

```xml
<?xml version="1.0" encoding="UTF-8"?>
<interface>
   <requires lib="gtk" version="4.0" />
   <object class="GtkWindow" id="window1">
      <property name="title">My GTK App!</property>
      <child>
         <object class="GtkBox">
            <property name="homogeneous">1</property>
            <property name="visible">1</property>
            <child>
               <object class="GtkLabel">
                  <property name="label">Hello, world!</property>
                  <property name="visible">1</property>
                  <property name="margin-top">10</property>
                  <property name="margin-bottom">10</property>
               </object>
            </child>
         </object>
      </child>
   </object>
</interface>
```

This is not a very readable or compact way to represent a UI. It is redundant and extremely verbose. Now lets look at what it could look like in GtkUI syntax.

```scss
#include "gtk-4.0"

@root {
  GtkWindow {
    GtkBox {
      GtkLabel("Hello, world!")
        .visible(true)
        .align("center")
        .margin-top(10)
        .margin-bottom(10)
    }
    .homogeneous(true)
    .visible(true)
  }
  .id("window1")
  .title("My GTK App!")
}
```

Not only is this less code, but it more accurately conveys the structure of the UI. You can easily tell what the object's children are, what properties it has, and more important information, like the text in a GtkLabel, is displayed right next to the object so you can get a pretty good idea of where you are when skimming the code. Lets try to add another GtkLabel to our box.

```xml
// ...

<child>
   <object class="GtkLabel">
      <property name="label">Hello, world!</property>
      <property name="visible">1</property>
      <property name="margin-top">10</property>
      <property name="margin-bottom">10</property>
   </object>
   
  // Add this

  <object class="GtkLabel">
    <property name="label">My Second Label</property>
    <property name="visible">1</property>
  </object>

  // -------
</child>

// ...
```

Not to belabor the point, but that's 6 lines for the bare minimum of a label. If we look at some GtkUI code...

```scss
// ...

GtkBox {
  GtkLabel("Hello, world!")
    .visible(true)
    .align("center")
    .margin-top(10)
    .margin-bottom(10)

  // Add this

  GtkLabel("My Second Label")
    .visible(true)

  // -------
}

// ...
```

its easily readable and efficient with space.
