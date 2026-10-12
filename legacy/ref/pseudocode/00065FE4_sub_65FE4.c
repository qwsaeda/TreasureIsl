/* ea:   0x00065FE4 */
/* name: sub_65FE4 */
/* size: 120 */
int __fastcall sub_65FE4(int a1, char *a2)
{
  int v4; // r0
  int result; // r0

  if ( a2 == nullptr )
    return sub_1309C(a1 + 4);
  v4 = sub_12F04(a2);
  sub_1318C((void **)(a1 + 4), a2, (int)&a2[2 * v4]);
  result = *(unsigned __int16 *)(*(_DWORD *)(a1 + 4) + 2 * ((*(_DWORD *)(a1 + 8) - *(_DWORD *)(a1 + 4)) >> 1) - 2) << 16;
  if ( *(_WORD *)(*(_DWORD *)(a1 + 4) + 2 * ((*(_DWORD *)(a1 + 8) - *(_DWORD *)(a1 + 4)) >> 1) - 2) != 92 )
    return sub_6605C(a1 + 4, 92);
  return result;
}

