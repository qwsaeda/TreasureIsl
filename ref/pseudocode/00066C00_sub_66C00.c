/* ea:   0x00066C00 */
/* name: sub_66C00 */
/* size: 96 */
int __fastcall sub_66C00(_DWORD *a1)
{
  int v2; // r0

  if ( a1 == nullptr || a1[7] == 0 || a1[9] == 0 )
    return -2;
  v2 = *(_DWORD *)(a1[7] + 20);
  if ( v2 != 0 )
    sub_6843C(v2, a1);
  ((void (__fastcall *)(_DWORD, _DWORD))a1[9])(a1[10], a1[7]);
  a1[7] = 0;
  return 0;
}

