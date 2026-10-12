/* ea:   0x00045D7C */
/* name: sub_45D7C */
/* size: 60 */
int __fastcall sub_45D7C(int a1)
{
  int v2; // r1
  int result; // r0

  v2 = *(_DWORD *)(a1 + 4);
  if ( v2 != 0 )
    (*(void (__fastcall **)(_DWORD))(*(_DWORD *)v2 + 12))(*(_DWORD *)(a1 + 4));
  result = *(_DWORD *)(a1 + 28);
  if ( result != 0 )
    return sub_5AD54(*(_DWORD *)(result + 20));
  return result;
}

