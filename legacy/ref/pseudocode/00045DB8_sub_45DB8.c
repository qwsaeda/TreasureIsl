/* ea:   0x00045DB8 */
/* name: sub_45DB8 */
/* size: 60 */
int __fastcall sub_45DB8(int a1)
{
  int v2; // r1
  int result; // r0

  v2 = *(_DWORD *)(a1 + 4);
  if ( v2 != 0 )
    (*(void (__fastcall **)(_DWORD))(*(_DWORD *)v2 + 16))(*(_DWORD *)(a1 + 4));
  result = *(_DWORD *)(a1 + 28);
  if ( result != 0 )
    return sub_5AD7C(*(_DWORD *)(result + 20));
  return result;
}

