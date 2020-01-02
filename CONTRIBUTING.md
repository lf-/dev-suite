# Contributing

## Thanks!

First of all thank you! You being here means you want to help in some way and
that's greatly appreciated!

## Mailing List

Part of dogfooding dev-suite's tools means figuring out how to make contributing
easy. It's not expected to be easy for a while, but that's part of the fun;
solving really hard problems. However, it's not impossible and working towards
this is the end goal of this project. There are a few ways you can contribute.

Many of them involve interacting with the mailing list:
- Mailing list email address: dev-suite@googlegroups.com
- [Google Group][google]

## Commit Messages

Before jumping into all of the below ways you can contribute please note that
any commits being submitted must be in the form laid out in
[this blog post][blog_post]. In summary all commits must have the following
form:

1. Separate subject from body with a blank line
2. Limit the subject line to 50 characters
3. Capitalize the subject line
4. Do not end the subject line with a period
5. Use the imperative mood in the subject line
6. Wrap the body at 72 characters
7. Use the body to explain what and why vs. how

Commits not following this will be rejected though number seven as a requirement
is a bit more lax.

## Feature Requests

There's a lot of things dev-suite can be and while there already some ideas for
future work it's nice to hear what people want out of their tools. Feature
requests will not be considered issues until they've been actively accepted as
something to be worked on. With that in mind please feel free to send in your
requests to the [wish list topic][wish_list] in the mailing list.

## Filing Issues

Bugs are inevitable feature of all software. If you find a bug please file an
issue with ticket in your own fork and send an email to
[the issues topic][issues] in the mailing list with where it can be pulled from
and it will get integrated it into the repo. Comments added to issues should be
done the same way as well. If it's a feature request follow the above to send
that in. In the future finding a way to automate this part so that you can just
send an email and have it merge the issue or comment automatically would be
great.

## Adding code or sending in bug fixes

Much like filing issues send an email to the mailing list with [PR] and what
the PR is for in the subject and a link to a publicly accessible repo where the
commits can be pulled from for review. Discussion will happen on that thread
until changes are accepted. Please follow the Commit Messages guidelines when
doing this. The history is not going to be cluttered with those that don't
follow the rules above. Having a nice git log to work with is absolutely
critical. Please also do not send patches by email. Sending patches screws up
the history and email clients do all kinds of stuff with newlines and white
space. git was designed to work in a distributed manner. Just make the repo
public and your changes should be able to be pulled in to the main repo!

[blog_post]: https://chris.beams.io/posts/git-commit/
[google]: https://groups.google.com/forum/#!forum/dev-sute
[wish_list]: https://groups.google.com/d/topic/dev-suite/H62oYcV-mE4/discussion
[issues]: https://groups.google.com/d/msg/dev-suite/IJdllJGoqSA/q6-VVmE9BAAJ
