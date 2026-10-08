/* ea:   0x0001C9E8 */
/* name: sub_1C9E8 */
/* size: 52 */
void __fastcall sub_1C9E8(int a1)
{
  void *v2; // r4

  sub_43CC4(a1);
  v2 = *(void **)(a1 + 1112);
  if ( v2 != nullptr )
  {
    sub_45EFC(*(_DWORD *)(a1 + 1112));
    operator delete(v2);
    *(_DWORD *)(a1 + 1112) = 0;
  }
}

