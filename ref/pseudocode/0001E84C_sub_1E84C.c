/* ea:   0x0001E84C */
/* name: sub_1E84C */
/* size: 64 */
int __fastcall sub_1E84C(int a1, int a2)
{
  int v3; // r3
  int result; // r0

  if ( *(_BYTE *)(a1 + 4) == 0 )
    __debugbreak();
  v3 = sub_1E88C(*(_DWORD *)(a1 + 8), *(_DWORD *)(a1 + 12), a2);
  result = 0;
  if ( v3 != *(_DWORD *)(a1 + 12) )
    return v3;
  return result;
}

