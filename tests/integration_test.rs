extern crate mustdown;
use mustdown::Parser;

#[test]
fn test_mustdown() {
    let text = "
# Header 1
## Header 2
### Header 3
#### Header 4
##### Header 5
###### Header 6
- list1
- list2
* list1
* list2
1. list1
2. list2
1) list1
2) list2
>this is a quote

>this is a quote  \nanother one

```
code block
```
---
***
[image]:to_image
Normal text **aha**, another *one*, `inline code`, and [link](to_link), and ![image](to_image).
Don't forget [link][link] and ![image][image]
[link]:to_link
";
    let mut parser = Parser::new();
    let result = parser.parse(text);
    let expected = "<h1>Header 1</h1>
<h2>Header 2</h2>
<h3>Header 3</h3>
<h4>Header 4</h4>
<h5>Header 5</h5>
<h6>Header 6</h6>
<ul>
<li>list1</li>
<li>list2</li>
</ul>
<ul>
<li>list1</li>
<li>list2</li>
</ul>
<ol start=\"1\">
<li>list1</li>
<li>list2</li>
</ol>
<ol start=\"1\">
<li>list1</li>
<li>list2</li>
</ol>
<blockquote><p>
this is a quote
</p></blockquote>
<blockquote><p>
this is a quote  <br>another one
</p></blockquote>
<pre><code>
code block
</code></pre>
<hr>
<hr>

<p>
Normal text <strong>aha</strong>, another <em>one</em>, <code>inline code</code>, and <a href=\"to_link\">link</a>, and <img src=\"to_image\" alt=\"image\">.
</p>
<p>
Don't forget <a href=\"to_link\">link</a> and <img src=\"to_image\" alt=\"image\">
</p>

";
    assert_eq!(expected, result);
}
