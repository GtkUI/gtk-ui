# Notes

## Props and Args and the different types

There are two different ways to set a property and two different ways to receive the information necessary to set that property. The many differences between a Prop and an Arg is that Args are required and the syntax for passing this information along differs. An Arg is given in an ArgList `(true, 1, "hello")` which is placed at the end of the name of a defined widget. For example:

```scss
GtkLabel("Hello, world!")
```

A Prop is optional so setting one would look like this:

```scss
GtkWindow {
  ...
}
.id("window1")
```

Additionally, how these are represented in XML depends on how they are defined. There are two different kinds of Args (Inline Args, Child Args) and Props (Inline Props, Child Props). The inline variants appear in the parent node in XML. For example, the id Prop above is defined as an InlineProp and therefore will show up in XMl like this:

```xml
<object class="GtkWindow" id="window1">
  ...
</object>
```

The child variants appear as children in XML. For example, the Arg example above is a ChildArg and therefore will should up in XML like this:

```xml
<object class="GtkLabel">
  <property name="label">Hello, world!</property>
</object>
```

## Inheritance

The rule of inheritance is that **ONLY PROPS ARE INHERITED**. If one wishes to inherit a GtkWindow, but make the `id` Prop an Arg as well, they are welcome to do so. If one wishes to inherit a GtkLabel, the Arg which takes the text will not be inherited, and if desired must be manually implemented.

The idea behind this is that there are going to be a lot of object defintions inheriting other object definitions, and having to implement some kind of override syntax would ultimately cause a lot more confusion than necessary. Put simply, you don't have to set a Prop, but you have to set an Arg. If someone is making a whole new object definition inheriting another, odds are they are going to have a pretty good idea if a property will always need to be set.
