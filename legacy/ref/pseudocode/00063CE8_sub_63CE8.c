/* ea:   0x00063CE8 */
/* name: sub_63CE8 */
/* size: 160 */
int __fastcall sub_63CE8(int a1, void *a2, int a3, char a4)
{
  int v6; // [sp+0h] [bp-18h] BYREF

  if ( a2 == nullptr )
    __debugbreak();
  if ( a4 == 0 || *(_DWORD *)(a3 + 4) == *(_DWORD *)(a3 + 8) )
  {
    memcpy(a2, *(const void **)(a3 + 12), *(_DWORD *)(a3 + 8));
  }
  else
  {
    v6 = *(_DWORD *)(a3 + 4);
    if ( sub_66B04(a2, &v6, *(_DWORD *)(a3 + 12)) != 0 || v6 != *(_DWORD *)(a3 + 4) )
      return 0;
  }
  return 1;
}

