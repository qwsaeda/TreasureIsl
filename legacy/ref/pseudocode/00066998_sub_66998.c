/* ea:   0x00066998 */
/* name: sub_66998 */
/* size: 108 */
__int16 *__fastcall sub_66998(int a1, __int16 **a2, char *a3)
{
  __int16 **v5; // r0
  int v6; // r0

  if ( a3 == nullptr )
    __debugbreak();
  v5 = (__int16 **)(a1 + 4);
  if ( v5 != a2 )
    sub_17890((int)a2, *v5, v5[1]);
  v6 = sub_12F04(a3);
  sub_136F8(a2, a3, &a3[2 * v6]);
  return *a2;
}

