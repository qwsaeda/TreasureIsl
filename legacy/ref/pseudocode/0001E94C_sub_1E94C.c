/* ea:   0x0001E94C */
/* name: sub_1E94C */
/* size: 40 */
int __fastcall sub_1E94C(int a1)
{
  if ( *(_BYTE *)(a1 + 4) == 0 )
    __debugbreak();
  return (*(_DWORD *)(a1 + 12) - *(_DWORD *)(a1 + 8)) >> 4;
}

