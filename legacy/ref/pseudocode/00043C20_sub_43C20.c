/* ea:   0x00043C20 */
/* name: sub_43C20 */
/* size: 60 */
void __fastcall sub_43C20(_DWORD *a1)
{
  void **v2; // r4

  *a1 = &off_6F518;
  sub_43CC4(a1);
  v2 = (void **)(a1 + 244);
  sub_44AAC(v2);
  if ( *v2 != nullptr )
    free(*v2);
}

