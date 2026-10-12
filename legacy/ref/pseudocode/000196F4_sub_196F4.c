/* ea:   0x000196F4 */
/* name: sub_196F4 */
/* size: 20 */
void __fastcall sub_196F4(void **a1)
{
  void *v1; // r0

  v1 = *a1;
  if ( v1 != nullptr )
    free(v1);
}

