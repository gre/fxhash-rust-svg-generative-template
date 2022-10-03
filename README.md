barebone project of a rust + svg generator.

the rust generates an "intermediary" svg file that needs to be "rendered" on the web page with post processing.

the template does NOT include the processing step but adopts some conventions: for instance we use R,G,B as individual color channels to express the intensity of color to use, typically to simulate inks.

I may "deconstruct" more the template to reduce to a more minimal version and remove this "specifics". That said, **it's a template made by greweb for greweb so PLEASE use at your own risk.**
