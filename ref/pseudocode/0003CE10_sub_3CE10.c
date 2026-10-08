/* ea:   0x0003CE10 */
/* name: sub_3CE10 */
/* size: 100 */
int __fastcall sub_3CE10(int a1)
{
  void ***v2; // r5
  void **v3; // r4

  v2 = *(void ****)(a1 + 4);
  *(_DWORD *)a1 = &off_6DAF4;
  while ( v2 != *(void ****)(a1 + 8) )
  {
    v3 = *v2;
    if ( *v2 != nullptr )
    {
      sub_16B24(*v2);
      operator delete(v3);
    }
    ++v2;
  }
  sub_3D36C(a1 + 4, *(void **)(a1 + 4), *(void **)(a1 + 8));
  return sub_3D3AC(a1 + 4);
}

