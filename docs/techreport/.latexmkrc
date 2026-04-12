# Point latexmk at the shared style/def files in papers/ when available.
my $papers_dir = $ENV{'CLISCRAPE_TEXINPUTS'} // "$ENV{HOME}/dev/papers";
if (-d $papers_dir) {
    $ENV{'TEXINPUTS'} = "$papers_dir//:" . ($ENV{'TEXINPUTS'} // '');
}
$pdf_mode = 4;  # Use LuaLaTeX (OpenType font embedding)
