/* ea:   0x00042F94 */
/* name: sub_42F94 */
/* size: 80 */
int __fastcall sub_42F94(int a1)
{
  int v2; // r1
  int result; // r0

  v2 = *(_DWORD *)(a1 + 16);
  if ( v2 != 0 )
    return (*(int (__fastcall **)(_DWORD))(*(_DWORD *)v2 + 12))(*(_DWORD *)(a1 + 16));
  result = *(_DWORD *)(a1 + 20);
  if ( *(_DWORD *)(a1 + 20) != 0 )
    return (*(int (__fastcall **)(_DWORD))(**(_DWORD **)(a1 + 20) + 12))(*(_DWORD *)(a1 + 20));
  return result;
}

