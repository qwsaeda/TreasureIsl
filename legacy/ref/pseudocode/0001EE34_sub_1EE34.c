/* ea:   0x0001EE34 */
/* name: sub_1EE34 */
/* size: 52 */
int __fastcall sub_1EE34(int a1, _DWORD *a2)
{
  int v3; // r3

  v3 = sub_1EE84(a1, 1);
  if ( v3 != -16 )
  {
    *(_DWORD *)(v3 + 16) = *a2;
    *(_DWORD *)(v3 + 20) = a2[1];
  }
  return v3;
}

