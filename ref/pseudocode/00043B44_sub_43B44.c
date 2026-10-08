/* ea:   0x00043B44 */
/* name: sub_43B44 */
/* size: 56 */
int __fastcall sub_43B44(int a1)
{
  void *v2; // r4

  v2 = *(void **)(a1 + 1024);
  if ( v2 != nullptr )
  {
    sub_45EFC(*(_DWORD *)(a1 + 1024));
    operator delete(v2);
    *(_DWORD *)(a1 + 1024) = 0;
  }
  return sub_43CC4(a1);
}

