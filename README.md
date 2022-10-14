# GtkUI

A language designed to improve readability and the overall efficiency of Gtk app development.

This language is heavily inspired by Apple's SwiftUI syntax and is generally the inspiration for this project.

## What does this solve?

Take a look at this very basic "hello world".

![Hello World Gtk App](https://user-images.githubusercontent.com/42098470/191370177-d36f35b0-68ba-4d85-9be7-b169b2ecc06c.png)

Here is what you would have to write in the standard "builder file" to get this.

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

Not only is this less code, but it more accurately conveys the structure of the UI. You can easily tell what the object's children are, what properties it has, and more important information, like the text in a GtkLabel, is displayed right next to the object so you can get a pretty good idea of where you are when skimming the code. Lets try to add another GtkLabel to our box. Here's what it should look like.

![Second Label Image](https://user-images.githubusercontent.com/42098470/191373277-63cf07fa-7901-4aa6-8496-29eef0c82a91.png)

And here are the changes that would need to be made to the builder file.

```xml
// ...

<object class="GtkBox">

  // ...

  // Add this

  <child>
    <object class="GtkLabel">
      <property name="label">My Second Label</property>
      <property name="visible">1</property>
    </object>
  </child>

</object>

// ...
```

Not to belabor the point, but that's a lot of code for the most basic of labels. If we look at some GtkUI code...

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
}

// ...
```

its easily readable and efficient with space.

