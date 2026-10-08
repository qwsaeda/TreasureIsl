/* ea:   0x000555A0 */
/* name: sub_555A0 */
/* size: 52 */
void __fastcall sub_555A0(_DWORD *a1)
{
  void *v2; // r4

  sub_43CC4(a1);
  v2 = (void *)a1[264];
  if ( v2 != nullptr )
  {
    sub_45EFC((_DWORD *)a1[264]);
    operator delete(v2);
    a1[264] = 0;
  }
}

