/* ea:   0x00067370 */
/* name: sub_67370 */
/* size: 156 */
int __fastcall sub_67370(int *a1, int a2, _DWORD *a3)
{
  int v5; // r0
  int v6; // r0
  int result; // r0

  if ( a3 != nullptr )
    *a3 = a1[15];
  v5 = *a1;
  if ( *a1 == 4 || v5 == 5 )
    (*(void (__fastcall **)(_DWORD, int))(a2 + 36))(*(_DWORD *)(a2 + 40), a1[3]);
  if ( *a1 == 6 )
    sub_68FD4(a1[1], a2);
  *a1 = 0;
  a1[7] = 0;
  a1[8] = 0;
  v6 = a1[10];
  a1[12] = v6;
  a1[13] = v6;
  result = a1[14];
  if ( result != 0 )
  {
    result = ((int (__fastcall *)(_DWORD, _DWORD, _DWORD))a1[14])(0, 0, 0);
    a1[15] = result;
    *(_DWORD *)(a2 + 48) = result;
  }
  return result;
}

