/* ea:   0x00038030 */
/* name: sub_38030 */
/* size: 80 */
int __fastcall sub_38030(_DWORD **a1, _DWORD *a2)
{
  _DWORD *v3; // r1
  int v4; // r5

  v3 = *a1;
  v4 = 0;
  while ( v3 != (_DWORD *)*a2 )
  {
    v3 = sub_16A30(v3);
    ++v4;
  }
  return v4;
}

