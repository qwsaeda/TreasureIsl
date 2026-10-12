/* ea:   0x00033E20 */
/* name: sub_33E20 */
/* size: 28 */
int sub_33E20(int a1, ...)
{
  va_list varg_r1; // [sp+Ch] [bp-Ch] BYREF

  va_start(varg_r1, a1);
  return sub_2F6A8(a1 + 16, (int *)varg_r1);
}

