/* ea:   0x0003BFD0 */
/* name: sub_3BFD0 */
/* size: 164 */
void *__fastcall sub_3BFD0(int a1, int a2, unsigned __int16 a3, _DWORD *a4)
{
  unsigned __int8 v8; // r4
  unsigned __int8 v10; // [sp+4h] [bp-1Ch] BYREF

  v8 = rand() % 9 + 1;
  if ( v8 <= 9u )
  {
    v10 = v8 - 1;
    *(_BYTE *)sub_318D0(a4 + 44, &v10) = 4;
  }
  sub_3AB44(a1, a2, a3, (int)a4, 1);
  return sub_34B38(a4, v8, 3);
}

